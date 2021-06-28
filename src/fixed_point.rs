use crate::demand::RequestBound;
use crate::supply::SupplyBound;
use crate::time::{Duration, Instant, Service};

use std::cmp::Ordering;

use thiserror::Error;

/// Error type returned when a fixed point search fails.
#[derive(Debug, Error, Copy, Clone, Eq, PartialEq, PartialOrd)]
pub enum SearchFailure {
    /// No fixed point found below the given divergence threshold.
    #[error("no fixed point less than {limit} found for offset {offset}")]
    DivergenceLimitExceeded { offset: Instant, limit: Duration },
}

pub type SearchResult = Result<Duration, SearchFailure>;

/// Conduct an iterative fixed point search up to a given divergence
/// threshold, assuming a given fixed `offset` within the busy
/// window.
pub fn search_with_offset<SBF, RHS>(
    supply: &SBF,
    offset: Instant,
    divergence_limit: Duration,
    workload: &RHS,
) -> SearchResult
where
    SBF: SupplyBound + ?Sized,
    RHS: Fn(Duration) -> Service,
{
    let mut assumed_response_time = 1;
    while assumed_response_time <= divergence_limit {
        let demand = workload(assumed_response_time);
        let response_time_bound = supply.service_time(demand) - offset;
        if response_time_bound <= assumed_response_time {
            // we have converged
            return Ok(response_time_bound);
        } else {
            // continue iterating
            assumed_response_time = response_time_bound
        }
    }
    // if we get here, we failed to converge => no solution
    Err(SearchFailure::DivergenceLimitExceeded {
        offset,
        limit: divergence_limit,
    })
}

/// Very slow, naive search for a fixed point up to the given
/// `divergence_limit`, assuming a given fixed `offset` within the
/// busy window. Do not use --- use [search_with_offset] instead.
#[cfg(debug_assertions)]
fn brute_force_search_with_offset<SBF, RHS>(
    supply: &SBF,
    offset: Instant,
    divergence_limit: Duration,
    workload: &RHS,
) -> SearchResult
where
    SBF: SupplyBound + ?Sized,
    RHS: Fn(Duration) -> Service,
{
    for r in 1..=divergence_limit {
        let lhs = supply.provided_service(offset + r);
        let rhs = workload(r);
        // corner case: zero demand is trivially satisfied immediately
        if rhs == 0 {
            return Ok(0);
        } else if lhs == rhs {
            return Ok(r);
        }
    }
    Err(SearchFailure::DivergenceLimitExceeded {
        offset,
        limit: divergence_limit,
    })
}

/// Iterative search for a fixed point up to a given
/// `divergence_limit`, assuming a given processor supply and a
/// generic workload bound.
pub fn search<SBF, RHS>(
    supply: &SBF,
    divergence_limit: Duration,
    workload_bound: RHS,
) -> SearchResult
where
    SBF: SupplyBound + ?Sized,
    RHS: Fn(Duration) -> Service,
{
    let bw = search_with_offset(supply, 0, divergence_limit, &workload_bound);
    // In debug mode, compare against the brute-force solution.
    #[cfg(debug_assertions)]
    debug_assert_eq!(
        brute_force_search_with_offset(supply, 0, divergence_limit, &workload_bound),
        bw
    );
    bw
}

/// Given a sequence of [SearchResult]s, either return the maximum
/// finite result (if no divergence errors occurred) or propagate the
/// first error encountered.
fn max_response_time(rta_per_offset: impl Iterator<Item = SearchResult>) -> SearchResult {
    rta_per_offset
        .max_by(|a, b| {
            // propagate any errors values
            if a.is_err() {
                // if a is an error, we want to report it
                Ordering::Greater
            } else if b.is_err() {
                // if a is not an error, but b is, then we want b
                Ordering::Less
            } else {
                // if neither is an error, report the maximum result
                a.unwrap().cmp(&b.unwrap())
            }
        })
        // If we have no result at all, there are no demand steps, so the
        // response-time is trivially zero.
        .unwrap_or(Ok(0))
}

/// Try to find a response-time bound for a given processor supply
/// model and a given processor demand model.
///
/// The search for a fixed point will be aborted if the given
/// divergence threshold indicated by `limit` is reached.
///
/// The fixed-point search relies on three relevant characterizations
/// of processor demand:
/// - `demand` is demand model from which all points are inferred at
///   which the demand curve exhibits "steps".
/// - `bw_demand_bound` is the right-hand side of the fixed-point
///   equation describing the maximum busy-window length, i.e., the
///   demand of "everything".
/// - `offset_demand_bound` is the right-hand side of the fixed-point
///   equation describing the response time for a given offset.
pub fn bound_response_time<SBF, RBF, F, G>(
    supply: &SBF,
    demand: &RBF,
    bw_demand_bound: F,
    offset_demand_bound: G,
    limit: Duration,
) -> SearchResult
where
    SBF: SupplyBound + ?Sized,
    RBF: RequestBound + ?Sized,
    F: Fn(Duration) -> Service,
    G: Fn(Instant, Duration) -> Service,
{
    // find a bound on the maximum busy-window
    let max_bw = search(supply, limit, bw_demand_bound)?;
    // Look at all points where the demand curve "steps".
    // Note that steps_iter() yields interval lengths, but we are interested in
    // offsets. Since the length of an interval [0, A] is A+1, we need to subtract one
    // to obtain the offset.
    let offsets = demand
        .steps_iter()
        .map(|x| x - 1)
        .take_while(|x| *x <= max_bw);
    // for each relevant offset in the search space,
    let rta_bounds = offsets.map(|offset| {
        let rhs = |delta| offset_demand_bound(offset, delta);
        let rta = search_with_offset(supply, offset, limit, &rhs);
        // In debug mode, compare against the brute-force solution.
        debug_assert_eq!(
            brute_force_search_with_offset(supply, offset, limit, &rhs),
            rta
        );
        rta
    });
    max_response_time(rta_bounds)
}
