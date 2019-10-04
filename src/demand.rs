use crate::arrivals::ArrivalBound;
use crate::time::Duration;

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

impl<'a, B: ArrivalBound + 'a> AsRef<dyn RequestBound + 'a> for WorstCaseRBF<B> {
    fn as_ref<'b>(&'b self) ->  &'b (dyn RequestBound +'a) {
        self
    }
}

impl<T: AsRef<dyn RequestBound>> RequestBound for Vec<T> {

    fn service_needed(&self, delta: Duration) -> Duration {
        self.iter().map(|rbf| rbf.as_ref().service_needed(delta)).sum()
    }

    fn max_single_job_cost(&self) -> Duration {
        self.iter()
            .map(|rbf| rbf.as_ref().max_single_job_cost())
            .max()
            .unwrap_or(0)
    }

    fn steps_iter<'a>(&'a self) -> Box<dyn Iterator<Item = Duration> + 'a> {
        Box::new(self.iter().map(|rbf| rbf.as_ref().steps_iter()).kmerge().dedup())
    }

    fn job_cost_iter<'a>(&'a self, delta: Duration) -> Box<dyn Iterator<Item = Duration> + 'a> {
        Box::new(self.iter().map(|rbf| rbf.as_ref().job_cost_iter(delta)).kmerge())
    }
}

impl<T: AsRef<dyn RequestBound>> AggregateRequestBound for Vec<T> {
    fn service_needed_by_n_jobs_per_component(&self, delta: Duration, max_jobs: usize) -> Duration {
        self.iter()
            .map(|rbf| rbf.as_ref().service_needed_by_n_jobs(delta, max_jobs))
            .sum()
    }
}