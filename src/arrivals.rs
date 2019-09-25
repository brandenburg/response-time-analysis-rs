use std::collections::VecDeque;
use std::iter;

use crate::time::{Duration, Instant};

pub trait ArrivalBound {
    fn number_arrivals(&self, delta: Duration) -> u64;

    fn maximal_trace(&self, horizon: Instant) -> Vec<Instant> {
        let mut arrival_trace = Vec::new();

        for delta in 1..=horizon {
            let n = self.number_arrivals(delta);
            while n as usize > arrival_trace.len() {
                arrival_trace.push(delta - 1)
            }
        }

        arrival_trace
    }

    fn steps_iter<'a>(&'a self) -> Box<dyn Iterator<Item = Duration> + 'a> {
        Box::new(
            (1..).filter(move |delta| {
                self.number_arrivals(*delta) > self.number_arrivals(*delta - 1)
            }),
        )
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Periodic {
    pub period: Duration,
}

fn divide_with_ceil(a: u64, b: u64) -> u64 {
    a / b + if a % b > 0 { 1 } else { 0 }
}

impl ArrivalBound for Periodic {
    fn number_arrivals(&self, delta: Duration) -> u64 {
        divide_with_ceil(delta, self.period)
    }

    fn steps_iter<'a>(&'a self) -> Box<dyn Iterator<Item = Duration> + 'a> {
        Box::new((0..).map(move |j| j * self.period + 1))
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Sporadic {
    pub min_inter_arrival: Duration,
    pub jitter: Duration,
}

impl ArrivalBound for Sporadic {
    fn number_arrivals(&self, delta: Duration) -> u64 {
        if delta > 0 {
            divide_with_ceil(delta + self.jitter, self.min_inter_arrival)
        } else {
            0
        }
    }

    fn steps_iter<'a>(&'a self) -> Box<dyn Iterator<Item = Duration> + 'a> {
        Box::new(
            iter::once(1).chain(
                (1..)
                    .filter(move |j| j * self.min_inter_arrival + 1 > self.jitter)
                    .map(move |j| j * self.min_inter_arrival + 1 - self.jitter),
            ),
        )
    }
}

impl From<Periodic> for Sporadic {
    fn from(p: Periodic) -> Self {
        Sporadic {
            min_inter_arrival: p.period,
            jitter: 0,
        }
    }
}

#[derive(Clone, Debug)]
pub struct CurvePrefix {
    min_distance: Vec<Duration>,
}

impl CurvePrefix {
    pub fn unroll_sporadic(s: &Sporadic, interval: Duration) -> CurvePrefix {
        let n = s.number_arrivals(interval) as usize + 1;
        let mut v = Vec::with_capacity(n);
        for i in 0..n {
            let periods = i as u64 + 1;
            if s.jitter >= periods * s.min_inter_arrival {
                v.push(0)
            } else {
                v.push(periods * s.min_inter_arrival - s.jitter)
            }
        }
        CurvePrefix { min_distance: v }
    }

    pub fn from_trace<'a>(
        arrival_times: impl Iterator<Item = &'a Instant>,
        prefix_jobs: usize,
    ) -> CurvePrefix {
        let mut d = Vec::with_capacity(prefix_jobs);
        let mut window: VecDeque<u64> = VecDeque::with_capacity(prefix_jobs + 1);

        // consider all job arrivals in the trace
        for t in arrival_times {
            // sanity check: the arrival times must be monotonic
            assert!(t >= window.back().unwrap_or(&t));
            // look at all arrival times in the sliding window, in order
            // from most recent to oldest
            for (i, v) in window.iter().rev().enumerate() {
                // compute the separation from the current arrival t to the arrival
                // of the (i + 1)-th preceding job
                let observed_gap = t - v;
                if d.len() <= i {
                    // we have not yet seen (i + 1) jobs in a row -> first sample
                    d.push(observed_gap)
                } else {
                    // update belief if we have seen something of less separation
                    // than previously
                    d[i] = d[i].min(observed_gap)
                }
            }
            // add arrival time to sliding window
            window.push_back(*t);
            // trim sliding window if necessary
            if window.len() > prefix_jobs {
                window.pop_front();
            }
        }
        for (i, x) in d.iter().enumerate() {
            println!("d[{}] = {}", i, x)
        }

        CurvePrefix { min_distance: d }
    }

    fn min_job_separation(&self) -> Duration {
        // minimum separation of two jobs given by first element
        self.min_distance[0]
    }

    fn largest_known_distance(&self) -> Duration {
        *self.min_distance.last().unwrap()
    }

    fn jobs_in_largest_known_distance(&self) -> u64 {
        self.min_distance.len() as u64
    }

    fn lookup_arrivals(&self, delta: Duration) -> u64 {
        // for really large vectors, this should be a binary search...
        for (i, distance) in self.min_distance.iter().enumerate() {
            if *distance + 1 > delta {
                return (i as u64) + 1;
            }
        }
        self.jobs_in_largest_known_distance() + 1
    }
}

impl ArrivalBound for CurvePrefix {
    fn number_arrivals(&self, delta: Duration) -> u64 {
        if delta > 0 {
            // first, resolve long delta by super-additivity of arrival curves
            let prefix = delta / self.largest_known_distance();
            let prefix_jobs = prefix * self.jobs_in_largest_known_distance();
            let tail = delta % self.largest_known_distance();
            if tail > self.min_job_separation() {
                prefix_jobs + dbg!(self.lookup_arrivals(tail))
            } else {
                prefix_jobs + dbg!((tail > 0) as u64)
            }
        } else {
            0
        }
    }

    fn steps_iter<'a>(&'a self) -> Box<dyn Iterator<Item = Duration> + 'a> {
        let diffs: Vec<_> = iter::once(0)
            .chain(self.min_distance.iter().copied())
            .zip(self.min_distance.iter().copied())
            .map(|(a, b)| b - a)
            .filter(|d| *d > 0)
            .collect();

        struct StepsIter {
            sum: Instant,
            step_sizes: Vec<Duration>,
            idx: usize
        }

        impl Iterator for StepsIter {
            type Item = Duration;

            fn next(&mut self) -> Option<Self::Item> {
                let val = self.sum;
                self.sum += self.step_sizes[self.idx];
                self.idx = (self.idx + 1) % self.step_sizes.len();
                Some(val)
            }
        }

        Box::new(StepsIter{sum: 1, step_sizes: diffs, idx: 0})
    }
}

impl From<Periodic> for CurvePrefix {
    fn from(p: Periodic) -> Self {
        CurvePrefix {
            min_distance: vec![p.period],
        }
    }
}

impl From<Sporadic> for CurvePrefix {
    fn from(s: Sporadic) -> Self {
        let jitter_jobs = divide_with_ceil(s.jitter, s.min_inter_arrival);
        // Jitter can cause pessimism when applying super-additivity.
        // Memory is cheap. Hence, unroll quite a bit to avoid running into
        // pessimism too frequently.
        // By default, unroll until the jitter jobs are no more than 10% of the
        // jobs of the jobs in the unrolled interval, and until for at least 500 jobs.
        let n = 500.max(jitter_jobs * 10);
        CurvePrefix::unroll_sporadic(&s, n * s.min_inter_arrival)
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Poisson {
    pub rate: f64,
}

impl Poisson {
    pub fn arrival_probability(&self, delta: Duration, njobs: u32) -> f64 {
        // quick and dirty factorial: k!
        let mut denominator = 1.0;
        for x in 1..(njobs + 1) {
            denominator *= x as f64;
        }
        let mean = delta as f64 * self.rate;
        let mut numerator = (-mean).exp(); // e^(- rate * delta)
        numerator *= mean.powi(njobs as i32); // (rate * delta)**k
        numerator / denominator
    }

    pub fn approximate(&self, epsilon: f64) -> ApproximatedPoisson {
        ApproximatedPoisson {
            poisson: self.clone(),
            epsilon,
        }
    }
}

pub struct ApproximatedPoisson {
    poisson: Poisson,
    epsilon: f64,
}

impl ArrivalBound for ApproximatedPoisson {
    fn number_arrivals(&self, delta: Duration) -> u64 {
        if delta > 0 {
            let mut cumulative_prob = 0.0;
            let mut njobs = 0;
            loop {
                cumulative_prob += self.poisson.arrival_probability(delta, njobs);
                if cumulative_prob + self.epsilon >= 1.0 {
                    break;
                }
                njobs += 1;
            }
            njobs as u64
        } else {
            0
        }
    }
}
