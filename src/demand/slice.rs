use itertools::Itertools;

use super::{AggregateRequestBound, RequestBound};
use crate::time::{Duration, Service};

/// Analogously to [Aggregate][super::Aggregate], a wrapper type for
/// representing the total demand of individual demand sources.
/// Whereas [Aggregate][super::Aggregate] wraps a vector, this type
/// wraps a slice of an array.
#[derive(Clone, Debug)]
pub struct Slice<'a, T> {
    slice: &'a [T],
}

impl<'a, T> Slice<'a, T> {
    pub fn of(slice: &'a [T]) -> Self {
        Slice { slice }
    }
}

impl<'a, T: RequestBound> RequestBound for Slice<'a, T> {
    fn service_needed(&self, delta: Duration) -> Service {
        self.slice.iter().map(|rbf| rbf.service_needed(delta)).sum()
    }

    fn least_wcet_in_interval(&self, delta: Duration) -> Service {
        self.slice
            .iter()
            .map(|rbf| rbf.least_wcet_in_interval(delta))
            .min()
            .unwrap_or_else(Service::none)
    }

    fn steps_iter<'b>(&'b self) -> Box<dyn Iterator<Item = Duration> + 'b> {
        Box::new(
            self.slice
                .iter()
                .map(|rbf| rbf.steps_iter())
                .kmerge()
                .dedup(),
        )
    }

    fn job_cost_iter<'b>(&'b self, delta: Duration) -> Box<dyn Iterator<Item = Service> + 'b> {
        Box::new(
            self.slice
                .iter()
                .map(|rbf| rbf.job_cost_iter(delta))
                .kmerge(),
        )
    }
}

impl<'a, T: RequestBound> AggregateRequestBound for Slice<'a, T> {
    fn service_needed_by_n_jobs_per_component(&self, delta: Duration, max_jobs: usize) -> Service {
        self.slice
            .iter()
            .map(|rbf| rbf.service_needed_by_n_jobs(delta, max_jobs))
            .sum()
    }
}
