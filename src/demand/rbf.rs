use crate::arrivals::ArrivalBound;
use crate::time::{Duration, Service};
use crate::wcet::JobCostModel;

use super::RequestBound;

/// The canonical request-bound function (RBF), which connects
/// arrival bounds and job-cost models.
///
/// Given an arrival model and a job-cost model, the RBF bounds
/// demand over an interval of length `delta` simply as the total
/// cumulative cost of the maximum number of jobs that can arrive in
/// an interval of length `delta`.
#[derive(Clone, Debug)]
pub struct RBF<B: ArrivalBound, C: JobCostModel> {
    pub wcet: C,
    pub arrival_bound: B,
}

impl<B: ArrivalBound, C: JobCostModel> RequestBound for RBF<B, C> {
    fn service_needed(&self, delta: Duration) -> Service {
        self.wcet
            .cost_of_jobs(self.arrival_bound.number_arrivals(delta))
    }

    fn least_wcet_in_interval(&self, delta: Duration) -> Service {
        self.wcet
            .least_wcet(self.arrival_bound.number_arrivals(delta))
    }

    fn steps_iter<'a>(&'a self) -> Box<dyn Iterator<Item = Duration> + 'a> {
        self.arrival_bound.steps_iter()
    }

    fn job_cost_iter<'a>(&'a self, delta: Duration) -> Box<dyn Iterator<Item = Service> + 'a> {
        Box::new(
            self.wcet
                .job_cost_iter()
                .take(self.arrival_bound.number_arrivals(delta)),
        )
    }
}

impl<'a, B: ArrivalBound + 'a, C: JobCostModel + 'a> AsRef<dyn RequestBound + 'a> for RBF<B, C> {
    fn as_ref<'b>(&'b self) -> &'b (dyn RequestBound + 'a) {
        self
    }
}
