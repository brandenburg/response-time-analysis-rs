use crate::arrivals::ArrivalBound;
use crate::supply::SupplyBound;
use crate::time::{Duration, Instant};

use std::cmp::Ordering;

use itertools::Itertools;

pub trait RequestBound {
    fn service_needed(&self, delta: Duration) -> Duration {
        self.job_cost_iter(delta).sum()
    }

    fn service_needed_by_n_jobs(&self, delta: Duration, max_jobs: usize) -> Duration {
        // take the max_jobs largest job costs
        itertools::sorted(self.job_cost_iter(delta))
            .rev()
            .take(max_jobs)
            .sum()
    }

    fn max_single_job_cost(&self) -> Duration {
        self.service_needed_by_n_jobs(1, 1)
    }

    fn steps_iter<'a>(&'a self) -> Box<dyn Iterator<Item = Duration> + 'a>;

    fn job_cost_iter<'a>(&'a self, delta: Duration) -> Box<dyn Iterator<Item = Duration> + 'a>;
}

pub trait AggregateRequestBound: RequestBound {
    fn service_needed_by_n_jobs_per_component(&self, delta: Duration, max_jobs: usize) -> Duration;
}

#[derive(Clone, Debug)]
pub struct WorstCaseRBF<B: ArrivalBound> {
    pub wcet: Duration,
    pub arrival_bound: B,
}

impl<B: ArrivalBound> RequestBound for WorstCaseRBF<B> {
    fn service_needed(&self, delta: Duration) -> Duration {
        self.arrival_bound.number_arrivals(delta) * self.wcet
    }

    fn service_needed_by_n_jobs(&self, delta: Duration, max_jobs: usize) -> Duration {
        self.arrival_bound
            .number_arrivals(delta)
            .min(max_jobs as u64)
            * self.wcet
    }

    fn max_single_job_cost(&self) -> Duration {
        self.wcet
    }

    fn steps_iter<'a>(&'a self) -> Box<dyn Iterator<Item = Duration> + 'a> {
        self.arrival_bound.steps_iter()
    }

    fn job_cost_iter<'a>(&'a self, delta: Duration) -> Box<dyn Iterator<Item = Duration> + 'a> {
        Box::new(itertools::repeat_n(
            self.wcet,
            self.arrival_bound.number_arrivals(delta) as usize,
        ))
    }
}

impl<T: RequestBound> RequestBound for Vec<T> {
    fn service_needed(&self, delta: Duration) -> Duration {
        self.iter().map(|rbf| rbf.service_needed(delta)).sum()
    }

    fn max_single_job_cost(&self) -> Duration {
        self.iter()
            .map(|rbf| rbf.max_single_job_cost())
            .max()
            .unwrap_or(0)
    }

    fn steps_iter<'a>(&'a self) -> Box<dyn Iterator<Item = Duration> + 'a> {
        Box::new(self.iter().map(|rbf| rbf.steps_iter()).kmerge().dedup())
    }

    fn job_cost_iter<'a>(&'a self, delta: Duration) -> Box<dyn Iterator<Item = Duration> + 'a> {
        Box::new(self.iter().map(|rbf| rbf.job_cost_iter(delta)).kmerge())
    }
}

impl<T: RequestBound> AggregateRequestBound for Vec<T> {
    fn service_needed_by_n_jobs_per_component(&self, delta: Duration, max_jobs: usize) -> Duration {
        self.iter()
            .map(|rbf| rbf.service_needed_by_n_jobs(delta, max_jobs))
            .sum()
    }
}

impl RequestBound for Vec<Box<dyn RequestBound>> {
    fn service_needed(&self, delta: Duration) -> Duration {
        self.iter().map(|rbf| rbf.service_needed(delta)).sum()
    }

    fn max_single_job_cost(&self) -> Duration {
        self.iter()
            .map(|rbf| rbf.max_single_job_cost())
            .max()
            .unwrap_or(0)
    }

    fn steps_iter<'a>(&'a self) -> Box<dyn Iterator<Item = Duration> + 'a> {
        Box::new(self.iter().map(|rbf| rbf.steps_iter()).kmerge().dedup())
    }

    fn job_cost_iter<'a>(&'a self, delta: Duration) -> Box<dyn Iterator<Item = Duration> + 'a> {
        Box::new(self.iter().map(|rbf| rbf.job_cost_iter(delta)).kmerge())
    }
}

impl AggregateRequestBound for Vec<Box<dyn RequestBound>> {
    fn service_needed_by_n_jobs_per_component(&self, delta: Duration, max_jobs: usize) -> Duration {
        self.iter()
            .map(|rbf| rbf.service_needed_by_n_jobs(delta, max_jobs))
            .sum()
    }
}

pub fn fixed_point_search_with_offset<SBF, RHS>(
    supply: &SBF,
    offset: Instant,
    divergence_limit: Duration,
    workload: &RHS,
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
    SBF: SupplyBound,
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
    SBF: SupplyBound,
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
    SBF: SupplyBound,
    RBF: RequestBound,
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
