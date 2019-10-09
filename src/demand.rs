use crate::arrivals::ArrivalBound;
use crate::time::Duration;

use itertools::Itertools;
use std::collections::VecDeque;
use std::iter::{self, FromIterator};
use auto_impl::auto_impl;

#[auto_impl(&, Box, Rc)]
pub trait JobCostModel {
    fn cost_of_jobs(&self, n: usize) -> Duration {
        self.job_cost_iter().take(n).sum()
    }

    fn least_wcet(&self, n: usize) -> Duration {
        self.job_cost_iter().take(n).min().unwrap_or(0)
    }

    fn job_cost_iter<'a>(&'a self) -> Box<dyn Iterator<Item = Duration> + 'a>;
}

// single WCET value
impl JobCostModel for Duration {
    fn cost_of_jobs(&self, n: usize) -> Duration {
        n as Duration * self
    }

    fn least_wcet(&self, n: usize) -> Duration {
        if n > 0 { *self } else { 0 }
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

    fn least_wcet(&self, n: usize) -> Duration {
        self.iter().take(n).copied().min().unwrap_or(0)
    }
}

#[derive(Clone, Debug)]
pub struct CostFunction {
    wcet_of_n_jobs: Vec<Duration>,
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
        Box::new((1..).map(move |n| self.cost_of_jobs(n) - self.cost_of_jobs(n - 1)))
    }

    fn least_wcet(&self, n: usize) -> Duration {
        if n > 0 {
            let mut least = self.wcet_of_n_jobs[0];
            for i in 1..self.wcet_of_n_jobs.len().min(n) {
                least = least.min(self.wcet_of_n_jobs[i] - self.wcet_of_n_jobs[i - 1])
            }
            least
        } else {
            0
        }
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

        CostFunction {
            wcet_of_n_jobs: cost_of,
        }
    }
}

impl FromIterator<Duration> for CostFunction {
    fn from_iter<I: IntoIterator<Item = Duration>>(iter: I) -> CostFunction {
        let mut wcets: Vec<Duration> = iter.into_iter().collect();
        // ensure the cost function is monotonic
        for i in 1..wcets.len() {
            wcets[i] = wcets[i].max(wcets[i - 1]);
        }
        CostFunction {
            wcet_of_n_jobs: wcets,
        }
    }
}

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
        self.wcet.least_wcet(self.arrival_bound.number_arrivals(delta))
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
        self.iter()
            .map(|rbf| rbf.service_needed(delta))
            .sum()
    }

    fn least_wcet_in_interval(&self, delta: Duration) -> Duration {
        self.iter()
            .map(|rbf| rbf.least_wcet_in_interval(delta))
            .min()
            .unwrap_or(0)
    }

    fn steps_iter<'a>(&'a self) -> Box<dyn Iterator<Item = Duration> + 'a> {
        Box::new(
            self.iter()
                .map(|rbf| rbf.steps_iter())
                .kmerge()
                .dedup(),
        )
    }

    fn job_cost_iter<'a>(&'a self, delta: Duration) -> Box<dyn Iterator<Item = Duration> + 'a> {
        Box::new(
            self.iter()
                .map(|rbf| rbf.job_cost_iter(delta))
                .kmerge(),
        )
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
        self.iter()
            .map(|rbf| rbf.service_needed(delta))
            .sum()
    }

    fn least_wcet_in_interval(&self, delta: Duration) -> Duration {
        self.iter()
            .map(|rbf| rbf.least_wcet_in_interval(delta))
            .min()
            .unwrap_or(0)
    }

    fn steps_iter<'a>(&'a self) -> Box<dyn Iterator<Item = Duration> + 'a> {
        Box::new(
            self.iter()
                .map(|rbf| rbf.steps_iter())
                .kmerge()
                .dedup(),
        )
    }

    fn job_cost_iter<'a>(&'a self, delta: Duration) -> Box<dyn Iterator<Item = Duration> + 'a> {
        Box::new(
            self.iter()
                .map(|rbf| rbf.job_cost_iter(delta))
                .kmerge(),
        )
    }
}

impl<T: RequestBound> AggregateRequestBound for [T] {
    fn service_needed_by_n_jobs_per_component(&self, delta: Duration, max_jobs: usize) -> Duration {
        self.iter()
            .map(|rbf| rbf.service_needed_by_n_jobs(delta, max_jobs))
            .sum()
    }
}
