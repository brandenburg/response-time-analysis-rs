use crate::arrivals::ArrivalBound;
use crate::time::Duration;

use std::collections::VecDeque;
use std::iter::{self, FromIterator};
use itertools::Itertools;

pub trait JobCostModel {
    fn cost_of_jobs(&self, n: usize) -> Duration {
        self.job_cost_iter().take(n).sum()
    }

    fn job_cost_iter<'a>(&'a self) -> Box<dyn Iterator<Item = Duration> + 'a>;
}

// single WCET value
impl JobCostModel for Duration {
    fn cost_of_jobs(&self, n: usize) -> Duration {
        n as Duration * self
    }

    fn job_cost_iter<'a>(&'a self) -> Box<dyn Iterator<Item = Duration> + 'a> {
        Box::new(iter::repeat(*self))
    }
}

// multiple WCET values => multi-frame tasks
impl JobCostModel for Vec<Duration> {
    fn job_cost_iter<'a>(&'a self) -> Box<dyn Iterator<Item = Duration> + 'a> {
        Box::new(self.iter().copied().cycle())
    }
}

#[derive(Clone, Debug)]
pub struct CostFunction {
    wcet_of_n_jobs: Vec<Duration>
}

impl JobCostModel for CostFunction {
    fn cost_of_jobs(&self, n: usize) -> Duration {
        if self.wcet_of_n_jobs.len() > 0 && n > 0 {
            // resolve large 'n' by super-additivity of cost function
            let x = n / self.wcet_of_n_jobs.len();
            let y = n % self.wcet_of_n_jobs.len();
            let prefix = x as Duration * self.wcet_of_n_jobs[self.wcet_of_n_jobs.len() - 1];
            let suffix = if y > 0 {
                // -1 to account for zero-based indexing: offset 0 holds cost of 1 job
                self.wcet_of_n_jobs[y - 1]
            } else {
                0
            };
            prefix + suffix
        } else {
            0
        }
    }

    fn job_cost_iter<'a>(&'a self) -> Box<dyn Iterator<Item = Duration> + 'a> {
        Box::new((1..).map(move |n| self.cost_of_jobs(n) - self.cost_of_jobs(n -1)))
    }
}

impl CostFunction {
    pub fn from_trace<'a>(
        job_costs: impl Iterator<Item = &'a Duration>,
        max_n: usize,
    ) -> CostFunction {
        let mut cost_of = Vec::with_capacity(max_n);
        let mut window: VecDeque<u64> = VecDeque::with_capacity(max_n + 1);

        // consider all observed costs in the trace
        for c in job_costs {
            // add job cost to sliding window
            window.push_back(*c);
            // trim sliding window if necessary
            if window.len() > max_n {
                window.pop_front();
            }

            // look at all job costs in the sliding window and keep track of total cost
            let mut total_cost = 0;
            for (i, k) in window.iter().enumerate() {
                total_cost += k;
                if cost_of.len() <= i {
                    // we have not yet seen (i + 1) costs in a row -> first sample
                    cost_of.push(total_cost)
                } else {
                    // update total cost of (i+1) jobs
                    cost_of[i] = cost_of[i].max(total_cost)
                }
            }
        }

        CostFunction{ wcet_of_n_jobs: cost_of }
    }
}

impl FromIterator<Duration> for CostFunction {
    fn from_iter<I: IntoIterator<Item=Duration>>(iter: I) -> CostFunction {
        let mut wcets: Vec<Duration> = iter.into_iter().collect();
        // ensure the cost function is monotonic
        for i in 1..wcets.len() {
            wcets[i] = wcets[i].max(wcets[i - 1]);
        }
        CostFunction{wcet_of_n_jobs: wcets}
    }
}


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

