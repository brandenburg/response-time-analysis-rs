use auto_impl::auto_impl;
use itertools::Itertools;

use crate::arrivals::ArrivalBound;
use crate::time::Duration;

use crate::wcet::JobCostModel;

#[auto_impl(&, Box, Rc)]
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

    fn least_wcet_in_interval(&self, delta: Duration) -> Duration;

    fn steps_iter<'a>(&'a self) -> Box<dyn Iterator<Item = Duration> + 'a>;

    fn job_cost_iter<'a>(&'a self, delta: Duration) -> Box<dyn Iterator<Item = Duration> + 'a>;
}

#[auto_impl(&, Box, Rc)]
pub trait AggregateRequestBound: RequestBound {
    fn service_needed_by_n_jobs_per_component(&self, delta: Duration, max_jobs: usize) -> Duration;
}

#[derive(Clone, Debug)]
pub struct RBF<B: ArrivalBound, C: JobCostModel> {
    pub wcet: C,
    pub arrival_bound: B,
}

impl<B: ArrivalBound, C: JobCostModel> RequestBound for RBF<B, C> {
    fn service_needed(&self, delta: Duration) -> Duration {
        self.wcet
            .cost_of_jobs(self.arrival_bound.number_arrivals(delta))
    }

    fn least_wcet_in_interval(&self, delta: Duration) -> Duration {
        self.wcet
            .least_wcet(self.arrival_bound.number_arrivals(delta))
    }

    fn steps_iter<'a>(&'a self) -> Box<dyn Iterator<Item = Duration> + 'a> {
        self.arrival_bound.steps_iter()
    }

    fn job_cost_iter<'a>(&'a self, delta: Duration) -> Box<dyn Iterator<Item = Duration> + 'a> {
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

impl<T: RequestBound> RequestBound for Vec<T> {
    fn service_needed(&self, delta: Duration) -> Duration {
        self.iter().map(|rbf| rbf.service_needed(delta)).sum()
    }

    fn least_wcet_in_interval(&self, delta: Duration) -> Duration {
        self.iter()
            .map(|rbf| rbf.least_wcet_in_interval(delta))
            .min()
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

impl<T: RequestBound> RequestBound for [T] {
    fn service_needed(&self, delta: Duration) -> Duration {
        self.iter().map(|rbf| rbf.service_needed(delta)).sum()
    }

    fn least_wcet_in_interval(&self, delta: Duration) -> Duration {
        self.iter()
            .map(|rbf| rbf.least_wcet_in_interval(delta))
            .min()
            .unwrap_or(0)
    }

    fn steps_iter<'a>(&'a self) -> Box<dyn Iterator<Item = Duration> + 'a> {
        Box::new(self.iter().map(|rbf| rbf.steps_iter()).kmerge().dedup())
    }

    fn job_cost_iter<'a>(&'a self, delta: Duration) -> Box<dyn Iterator<Item = Duration> + 'a> {
        Box::new(self.iter().map(|rbf| rbf.job_cost_iter(delta)).kmerge())
    }
}

impl<T: RequestBound> AggregateRequestBound for [T] {
    fn service_needed_by_n_jobs_per_component(&self, delta: Duration, max_jobs: usize) -> Duration {
        self.iter()
            .map(|rbf| rbf.service_needed_by_n_jobs(delta, max_jobs))
            .sum()
    }
}
