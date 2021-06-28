use itertools::Itertools;

use super::{AggregateRequestBound, RequestBound};
use crate::time::{Duration, Service};

/// A wrapper type for representing the total demand of a vector of
/// individual demand sources (e.g., all higher-priority tasks).
#[derive(Clone, Debug)]
pub struct Aggregate<T> {
    individual: Vec<T>,
}

impl<T> Aggregate<T> {
    pub fn new(components: Vec<T>) -> Self {
        Aggregate {
            individual: components,
        }
    }
}

impl<T: RequestBound> RequestBound for Aggregate<T> {
    fn service_needed(&self, delta: Duration) -> Service {
        self.individual
            .iter()
            .map(|rbf| rbf.service_needed(delta))
            .sum()
    }

    fn least_wcet_in_interval(&self, delta: Duration) -> Service {
        self.individual
            .iter()
            .map(|rbf| rbf.least_wcet_in_interval(delta))
            .min()
            .unwrap_or(0)
    }

    fn steps_iter<'a>(&'a self) -> Box<dyn Iterator<Item = Duration> + 'a> {
        Box::new(
            self.individual
                .iter()
                .map(|rbf| rbf.steps_iter())
                .kmerge()
                .dedup(),
        )
    }

    fn job_cost_iter<'a>(&'a self, delta: Duration) -> Box<dyn Iterator<Item = Service> + 'a> {
        Box::new(
            self.individual
                .iter()
                .map(|rbf| rbf.job_cost_iter(delta))
                .kmerge(),
        )
    }
}

impl<T: RequestBound> AggregateRequestBound for Aggregate<T> {
    fn service_needed_by_n_jobs_per_component(&self, delta: Duration, max_jobs: usize) -> Service {
        self.individual
            .iter()
            .map(|rbf| rbf.service_needed_by_n_jobs(delta, max_jobs))
            .sum()
    }
}
