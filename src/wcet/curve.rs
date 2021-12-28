use std::cell::RefCell;
use std::collections::VecDeque;
use std::iter::FromIterator;
use std::rc::Rc;

use super::JobCostModel;
use crate::time::Service;

/// The WCET curve model of a task's cumulative execution-time
/// demand. Similar to arrival curves, the cost of `n` consecutive
/// jobs is specified by a vector of cumulative WCET for 1,2,3,...
/// jobs and extrapolated from there.
#[derive(Clone, Debug)]
pub struct Curve {
    wcet_of_n_jobs: Vec<Service>,
}

impl Curve {
    /// Create a new WCET curve model based on the given finite
    /// prefix of cumulative demand.
    ///
    /// The reference input `wcet_of_n_jobs` is assumed to start at
    /// one job, i.e., the first element of the vector (at offset
    /// zero) holds the maximum cost of any one job, the second
    /// element of the vector (at offset one) holds the cumulative
    /// cost of any two consecutive jobs, the third element of the
    /// vector (at offset two) holds the cumulative cost of any three
    /// consecutive jobs, etc.
    ///
    /// **Note**: the constructor currently doesn't check whether the
    /// input is well-formed, so garbage in => garbage out...
    pub fn new(wcet_of_n_jobs: Vec<Service>) -> Self {
        Curve { wcet_of_n_jobs }
    }
}

impl JobCostModel for Curve {
    fn cost_of_jobs(&self, n: usize) -> Service {
        if !self.wcet_of_n_jobs.is_empty() && n > 0 {
            // resolve large 'n' by super-additivity of cost function
            let x = n / self.wcet_of_n_jobs.len();
            let y = n % self.wcet_of_n_jobs.len();
            let prefix = if x > 0 {
                self.wcet_of_n_jobs[self.wcet_of_n_jobs.len() - 1] * x as u64
            } else {
                Service::none()
            };
            let suffix = if y > 0 {
                // -1 to account for zero-based indexing: offset 0 holds cost of 1 job
                self.wcet_of_n_jobs[y - 1]
            } else {
                Service::none()
            };
            prefix + suffix
        } else {
            Service::none()
        }
    }

    fn job_cost_iter<'a>(&'a self) -> Box<dyn Iterator<Item = Service> + 'a> {
        Box::new((1..).map(move |n| self.cost_of_jobs(n) - self.cost_of_jobs(n - 1)))
    }

    fn least_wcet(&self, n: usize) -> Service {
        if n > 0 {
            let mut least = self.wcet_of_n_jobs[0];
            for i in 1..self.wcet_of_n_jobs.len().min(n) {
                least = least.min(self.wcet_of_n_jobs[i] - self.wcet_of_n_jobs[i - 1])
            }
            least
        } else {
            Service::none()
        }
    }
}

impl Curve {
    pub fn from_trace(job_costs: impl Iterator<Item = Service>, max_n: usize) -> Curve {
        let mut cost_of = Vec::with_capacity(max_n);
        let mut window: VecDeque<Service> = VecDeque::with_capacity(max_n + 1);

        // consider all observed costs in the trace
        for c in job_costs {
            // add job cost to sliding window
            window.push_back(c);
            // trim sliding window if necessary
            if window.len() > max_n {
                window.pop_front();
            }

            // look at all job costs in the sliding window and keep track of total cost
            let mut total_cost = Service::none();
            for (i, k) in window.iter().enumerate() {
                total_cost += *k;
                if cost_of.len() <= i {
                    // we have not yet seen (i + 1) costs in a row -> first sample
                    cost_of.push(total_cost)
                } else {
                    // update total cost of (i+1) jobs
                    cost_of[i] = cost_of[i].max(total_cost)
                }
            }
        }

        Curve {
            wcet_of_n_jobs: cost_of,
        }
    }

    fn extrapolate_next(&self) -> Service {
        let n = self.wcet_of_n_jobs.len();
        assert!(n >= 2);
        // Upper-bound cost of n jobs as the sum of the bounds on the costs of
        // n-k jobs and k jobs. Since we don't store n=0, this is offset by one.
        (0..=(n / 2))
            .map(|k| self.wcet_of_n_jobs[k] + self.wcet_of_n_jobs[n - k - 1])
            .min()
            .unwrap()
    }

    pub fn extrapolate(&mut self, n: usize) {
        // We need at least three samples to extrapolate, so let's do nothing if we have fewer.
        if self.wcet_of_n_jobs.len() >= 3 {
            while self.wcet_of_n_jobs.len() < n - 1 {
                self.wcet_of_n_jobs.push(self.extrapolate_next())
            }
        }
    }
}

impl FromIterator<Service> for Curve {
    fn from_iter<I: IntoIterator<Item = Service>>(iter: I) -> Curve {
        let mut wcets: Vec<Service> = iter.into_iter().collect();
        // ensure the cost function is monotonic
        for i in 1..wcets.len() {
            wcets[i] = wcets[i].max(wcets[i - 1]);
        }
        Curve::new(wcets)
    }
}

/// A variant of [Curve] that uses interior mutability to cache extrapolations.
/// Functionality equivalent to [Curve], but faster on average if
/// extrapolation occurs often.
#[derive(Clone, Debug)]
pub struct ExtrapolatingCurve {
    prefix: Rc<RefCell<Curve>>,
}

impl ExtrapolatingCurve {
    pub fn new(costfn: Curve) -> Self {
        ExtrapolatingCurve {
            prefix: Rc::new(RefCell::new(costfn)),
        }
    }
}

impl JobCostModel for ExtrapolatingCurve {
    fn cost_of_jobs(&self, n: usize) -> Service {
        let mut costfn = self.prefix.borrow_mut();
        costfn.extrapolate(n + 1);
        costfn.cost_of_jobs(n)
    }

    fn job_cost_iter<'a>(&'a self) -> Box<dyn Iterator<Item = Service> + 'a> {
        Box::new((1..).map(move |n| self.cost_of_jobs(n) - self.cost_of_jobs(n - 1)))
    }

    fn least_wcet(&self, n: usize) -> Service {
        // Don't need to extrapolate for this; the least delta is
        // fully determined by the initial prefix.
        self.prefix.borrow().least_wcet(n)
    }
}
