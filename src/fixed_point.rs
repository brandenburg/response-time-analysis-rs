use std::cmp::Ordering;

use thiserror::Error;

use crate::supply::SupplyBound;
use crate::time::{Duration, Offset, Service, Time};

/// Error type returned when a fixed point search fails.
#[derive(Debug, Error, Copy, Clone, Eq, PartialEq, PartialOrd)]
pub enum SearchFailure {
    /// No fixed point found below the given divergence threshold.
    #[error("no fixed point less than {limit} found for offset {offset}")]
    DivergenceLimitExceeded { offset: Offset, limit: Duration },
}

pub type SearchResult = Result<Duration, SearchFailure>;

/// Conduct an iterative fixed point search up to a given divergence
/// threshold, assuming a given fixed `offset` within the busy
/// window.
pub fn search_with_offset<SBF, RHS>(
    supply: &SBF,
    offset: Offset,
    divergence_limit: Duration,
    workload: &RHS,
) -> SearchResult
where
    SBF: SupplyBound + ?Sized,
    RHS: Fn(Duration) -> Service,
{
    let mut assumed_response_time = Duration::from(1);
    while assumed_response_time <= divergence_limit {
        let demand = workload(assumed_response_time);
        let demand_met = Offset::from_time_zero(supply.service_time(demand));
        let response_time_bound = offset.distance_to(demand_met);
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
    offset: Offset,
    divergence_limit: Duration,
    workload: &RHS,
) -> SearchResult
where
    SBF: SupplyBound + ?Sized,
    RHS: Fn(Duration) -> Service,
{
    for r in 1..=Time::from(divergence_limit) {
        let assumed_response_time = Duration::from(r);
        let lhs = supply.provided_service(offset.since_time_zero() + assumed_response_time);
        let rhs = workload(assumed_response_time);
        // corner case: zero demand is trivially satisfied immediately
        if rhs.is_none() {
            return Ok(Duration::zero());
        } else if lhs == rhs {
            return Ok(assumed_response_time);
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
    let bw = search_with_offset(supply, Offset::from(0), divergence_limit, &workload_bound);
    // In debug mode, compare against the brute-force solution.
    #[cfg(debug_assertions)]
    debug_assert_eq!(
        brute_force_search_with_offset(supply, Offset::from(0), divergence_limit, &workload_bound),
        bw
    );
    bw
}

/// Given a sequence of [SearchResult]s, either return the maximum
/// finite result (if no divergence errors occurred) or propagate the
/// first error encountered.
///
/// This utility function is useful when analyzing a set of offsets
/// that must be considered as part of a response-time analysis.
pub fn max_response_time(rta_per_offset: impl Iterator<Item = SearchResult>) -> SearchResult {
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
        .unwrap_or(Ok(Duration::zero()))
}
