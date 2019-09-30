use crate::arrivals::ArrivalBound;
use crate::supply::SupplyBound;
use crate::time::{Duration, Instant};

use std::cmp::Ordering;

use itertools::Itertools;

pub trait RequestBound {
    fn service_needed(&self, delta: Duration) -> Duration;

    fn steps_iter<'a>(&'a self) -> Box<dyn Iterator<Item = Duration> + 'a>;
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

    fn steps_iter<'a>(&'a self) -> Box<dyn Iterator<Item = Duration> + 'a> {
        self.arrival_bound.steps_iter()
    }
}

pub struct JointRBF {
    components: Vec<Box<dyn RequestBound>>
}

impl<'a> JointRBF {
    pub fn new() -> JointRBF {
        JointRBF{ components: Vec::new() }
    }

    pub fn add_boxed(&mut self, rbf: Box<dyn RequestBound>) {
        self.components.push(rbf)
    }

    pub fn add<RBF: RequestBound + Clone + 'static>(&mut self, rbf: &RBF) {
        self.add_boxed(Box::new(rbf.clone()))
    }
}

impl RequestBound for JointRBF {
    fn service_needed(&self, delta: Duration) -> Duration {
        self.components.iter().map(|rbf| rbf.service_needed(delta)).sum()
    }

    fn steps_iter<'a>(&'a self) -> Box<dyn Iterator<Item = Duration> + 'a> {
        Box::new(self.components.iter().map(|rbf| rbf.steps_iter()).kmerge().dedup())
    }
}

pub fn fixed_point_search_with_offset<SBF, RHS>(
    supply: &SBF,
    offset: Instant,
    divergence_limit: Duration,
    workload: RHS,
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

pub fn fixed_point_search<SBF, RHS>(
    supply: &SBF,
    divergence_limit: Duration,
    workload_bound: RHS,
) -> Option<Duration>
where
    SBF: SupplyBound,
    RHS: Fn(Duration) -> Duration,
{
    fixed_point_search_with_offset(supply, 0, divergence_limit, workload_bound)
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

pub fn response_time_analysis<SBF, RBF, F, G>(
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
        let offsets = demand.steps_iter().map(|x| x - 1).take_while(|x| *x <= max_bw);
        // for each relevant offset in the search space, 
        let rta_bounds = offsets.map(|offset| {
            let rhs = |delta| offset_demand_bound(offset, delta);
            fixed_point_search_with_offset(supply, offset, limit, rhs)
        });
        max_response_time(rta_bounds)
    } else {
        // in case of an unbounded busy-window, we cannot report a reponse-time bound
        None
    }
}
