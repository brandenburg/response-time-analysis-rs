use super::JobCostModel;
use crate::time::Service;

/// The classic multi-frame model of job costs. The worst-case
/// execution time of jobs is specified as a vector of bounds.
/// Consecutive jobs cycle through the given vector of bounds.
#[derive(Debug, Clone)]
pub struct Multiframe {
    costs: Vec<Service>,
}

impl Multiframe {
    /// Construct a new multi-frame cost model by wrapping a given
    /// vector of WCET values.
    pub fn new(costs: Vec<Service>) -> Self {
        Multiframe { costs }
    }
}

impl JobCostModel for Multiframe {
    fn job_cost_iter<'a>(&'a self) -> Box<dyn Iterator<Item = Service> + 'a> {
        Box::new(self.costs.iter().copied().cycle())
    }

    fn least_wcet(&self, n: usize) -> Service {
        self.costs.iter().take(n).copied().min().unwrap_or(0)
    }
}
