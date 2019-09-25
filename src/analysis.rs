use crate::arrivals::ArrivalBound;
use crate::supply::SupplyBound;
use crate::time::{Duration, Instant};

pub trait RequestBound {
    fn service_needed(&self, delta: Duration) -> Duration;
}

pub struct FullWCET<B: ArrivalBound> {
    pub per_job_wcet: Duration,
    pub arrival_bound: B,
}

impl<B: ArrivalBound> RequestBound for FullWCET<B> {
    fn service_needed(&self, delta: Duration) -> Duration {
        self.arrival_bound.number_arrivals(delta) * self.per_job_wcet
    }
}

pub fn fixed_point_search<SBF, RHS>(
    supply: SBF,
    offset: Instant,
    divergence_limit: Duration,
    workload: RHS
) -> Option<Duration>
where
    SBF: SupplyBound,
    RHS: Fn(Duration) -> Duration,
{
    let mut assumed_response_time = 1;
    while assumed_response_time <= divergence_limit {
        let demand = workload(assumed_response_time);
        let response_time_bound = supply.service_time(demand) - offset;
        if response_time_bound <= assumed_response_time {
            // we have converged
            return Some(response_time_bound)
        } else {
            // continue iterating
            assumed_response_time = response_time_bound
        }
    }
    // if we get here, we failed to converge => no solution
    None
}
