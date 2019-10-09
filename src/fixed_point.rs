use crate::supply::SupplyBound;
use crate::demand::RequestBound;
use crate::time::{Duration, Instant};

use std::cmp::Ordering;

pub fn fixed_point_search_with_offset<SBF, RHS>(
    supply: &SBF,
    offset: Instant,
    divergence_limit: Duration,
    workload: &RHS,
) -> Option<Duration>
where
    SBF: SupplyBound + ?Sized,
    RHS: Fn(Duration) -> Duration,
{
    let mut assumed_response_time = 1;
    while assumed_response_time <= divergence_limit {
        let demand = workload(assumed_response_time);
        let response_time_bound = supply.service_time(demand) - offset;
        if response_time_bound <= assumed_response_time {
            // we have converged
            dbg!(offset);
            dbg!(assumed_response_time);
            dbg!(supply.provided_service(response_time_bound + offset));
            dbg!(workload(response_time_bound));
            dbg!(response_time_bound);
            return Some(response_time_bound);
        } else {
            // continue iterating
            assumed_response_time = response_time_bound
        }
    }
    // if we get here, we failed to converge => no solution
    None
}

pub fn brute_force_fixed_point_search_with_offset<SBF, RHS>(
    supply: &SBF,
    offset: Instant,
    divergence_limit: Duration,
    workload: &RHS,
) -> Option<Duration>
where
    SBF: SupplyBound + ?Sized,
    RHS: Fn(Duration) -> Duration,
{
    for r in 1..=divergence_limit {
        let lhs = supply.provided_service(offset + r);
        let rhs = workload(r);
        if lhs == rhs {
            dbg!((offset, r, lhs, rhs));
            return Some(r);
        }
    }
    None
}

pub fn fixed_point_search<SBF, RHS>(
    supply: &SBF,
    divergence_limit: Duration,
    workload_bound: RHS,
) -> Option<Duration>
where
    SBF: SupplyBound + ?Sized,
    RHS: Fn(Duration) -> Duration,
{
    let bw = fixed_point_search_with_offset(supply, 0, divergence_limit, &workload_bound);
    debug_assert_eq!(
        brute_force_fixed_point_search_with_offset(supply, 0, divergence_limit, &workload_bound),
        bw
    );
    bw
}

pub fn max_response_time(
    rta_per_offset: impl Iterator<Item = Option<Duration>>,
) -> Option<Duration> {
    rta_per_offset
        .max_by(|a, b| {
            // propagate any None values
            if a.is_none() {
                Ordering::Less
            } else if b.is_none() {
                Ordering::Greater
            } else {
                a.unwrap().cmp(&b.unwrap())
            }
        })
        .unwrap_or(None)
}

pub fn bound_response_time<SBF, RBF, F, G>(
    supply: &SBF,
    demand: &RBF,
    bw_demand_bound: F,
    offset_demand_bound: G,
    limit: Duration,
) -> Option<Duration>
where
    SBF: SupplyBound + ?Sized,
    RBF: RequestBound + ?Sized,
    F: Fn(Duration) -> Duration,
    G: Fn(Instant, Duration) -> Duration,
{
    // find a bound on the maximum busy-window
    if let Some(max_bw) = fixed_point_search(supply, limit, bw_demand_bound) {
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
            let rta = fixed_point_search_with_offset(supply, offset, limit, &rhs);
            debug_assert_eq!(
                brute_force_fixed_point_search_with_offset(supply, offset, limit, &rhs),
                rta
            );
            rta
        });
        max_response_time(rta_bounds)
    } else {
        // in case of an unbounded busy-window, we cannot report a reponse-time bound
        None
    }
}
