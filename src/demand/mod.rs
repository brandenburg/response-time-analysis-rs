use auto_impl::auto_impl;

use crate::time::{Duration, Service};

/// The general interface for (arbitrarily shaped) processor demand.
/// This can represent the demand of a single task, or the cumulative
/// demand of multiple tasks.
#[auto_impl(&, Box, Rc)]
pub trait RequestBound {
    /// Bound the total amount of service needed in an interval of length `delta`.
    fn service_needed(&self, delta: Duration) -> Service {
        self.job_cost_iter(delta).sum()
    }

    /// Bound the total amount of service needed by up to `max_jobs`
    /// in an interval of length `delta`.
    fn service_needed_by_n_jobs(&self, delta: Duration, max_jobs: usize) -> Service {
        // take the max_jobs largest job costs
        itertools::sorted(self.job_cost_iter(delta))
            .rev()
            .take(max_jobs)
            .sum()
    }

    /// Expose the smallest WCET of any job encountered in an interval of length `delta`.
    fn least_wcet_in_interval(&self, delta: Duration) -> Service;

    /// Yield an iterator over the points (i.e., values of `delta` in
    /// [RequestBound::service_needed]) at which the cumulative demand changes.
    fn steps_iter<'a>(&'a self) -> Box<dyn Iterator<Item = Duration> + 'a>;

    /// Expose an iterator over the individual costs that comprise the overall demand.
    fn job_cost_iter<'a>(&'a self, delta: Duration) -> Box<dyn Iterator<Item = Service> + 'a>;
}

/// A refined interface for processor demand that represents the
/// total demand of a collection of individual sources of demand.
#[auto_impl(&, Box, Rc)]
pub trait AggregateRequestBound: RequestBound {
    /// Bound the total amount of service needed in an interval of
    /// length `delta`, subject to the constraint that no individual
    /// source of demand contributes more than `max_jobs` worth of
    /// demand.
    fn service_needed_by_n_jobs_per_component(&self, delta: Duration, max_jobs: usize) -> Service;
}

mod aggregate;
mod rbf;
mod slice;

pub use aggregate::Aggregate;
pub use rbf::RBF;
pub use slice::Slice;
