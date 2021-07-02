use std::iter;

use super::JobCostModel;
use crate::time::Service;

/// The classic and most simple characterization of the worst-case
/// processor demand of a single job: the scalar WCET bound.
#[derive(Debug, Clone, Copy)]
pub struct Scalar {
    wcet: Service,
}

impl Scalar {
    /// Construct a new `Sclar` cost model by wrapping a given WCET bound.
    pub fn new(wcet: Service) -> Self {
        Scalar { wcet }
    }
}

impl From<Service> for Scalar {
    fn from(val: Service) -> Self {
        Self::new(val)
    }
}

impl JobCostModel for Scalar {
    fn cost_of_jobs(&self, n: usize) -> Service {
        self.wcet * n as u64
    }

    fn least_wcet(&self, n: usize) -> Service {
        if n > 0 {
            self.wcet
        } else {
            Service::none()
        }
    }

    fn job_cost_iter<'a>(&'a self) -> Box<dyn Iterator<Item = Service> + 'a> {
        Box::new(iter::repeat(self.wcet))
    }
}
