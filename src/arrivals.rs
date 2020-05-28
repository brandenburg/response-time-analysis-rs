use std::cell::RefCell;
use std::collections::VecDeque;
use std::iter::{self, FromIterator};

use itertools::Itertools;

use crate::time::{Duration, Instant};
use auto_impl::auto_impl;

#[auto_impl(&, Box, Rc)]
pub trait ArrivalBound {
    fn number_arrivals(&self, delta: Duration) -> usize;

    // The sequence of interval lengths for which the arrival bound "steps", i.e.,
    // where it shows an increase.
    fn steps_iter<'a>(&'a self) -> Box<dyn Iterator<Item = Duration> + 'a> {
        Box::new(
            (1..).filter(move |delta| {
                self.number_arrivals(*delta) > self.number_arrivals(*delta - 1)
            }),
        )
    }

    fn clone_with_jitter(&self, jitter: Duration) -> Box<dyn ArrivalBound>;
}

#[derive(Copy, Clone, Debug)]
pub struct Periodic {
    pub period: Duration,
}

fn divide_with_ceil(a: u64, b: u64) -> u64 {
    a / b + if a % b > 0 { 1 } else { 0 }
}

impl ArrivalBound for Periodic {
    fn number_arrivals(&self, delta: Duration) -> usize {
        divide_with_ceil(delta, self.period) as usize
    }

    fn steps_iter<'a>(&'a self) -> Box<dyn Iterator<Item = Duration> + 'a> {
        Box::new((0..).map(move |j| j * self.period + 1))
    }

    fn clone_with_jitter(&self, jitter: Duration) -> Box<dyn ArrivalBound> {
        let mut ab = Box::new(Sporadic::from(self.clone()));
        ab.jitter = jitter;
        ab
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Never {}

impl ArrivalBound for Never {
    fn number_arrivals(&self, _delta: Duration) -> usize {
        0
    }

    fn steps_iter<'a>(&'a self) -> Box<dyn Iterator<Item = Duration> + 'a> {
        Box::new(iter::empty())
    }

    fn clone_with_jitter(&self, _jitter: Duration) -> Box<dyn ArrivalBound> {
        Box::new(Never{})
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Sporadic {
    pub min_inter_arrival: Duration,
    pub jitter: Duration,
}

impl ArrivalBound for Sporadic {
    fn number_arrivals(&self, delta: Duration) -> usize {
        if delta > 0 {
            divide_with_ceil(delta + self.jitter, self.min_inter_arrival) as usize
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

    fn clone_with_jitter(&self, added_jitter: Duration) -> Box<dyn ArrivalBound> {
        let mut ab = Box::new(self.clone());
        ab.jitter += added_jitter;
        ab
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
    // Convention: we do not store the mininum distance for 0 and 1 jobs.
    // So the min_distance vector at offset 0 contains the minimum
    // separation of two jobs.
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
                // Compute the separation from the current arrival t to the arrival
                // of the (i + 1)-th preceding job.
                // So if i=0, we are looking at two adjacent jobs.
                let observed_gap = t - v;
                if d.len() <= i {
                    // we have not yet seen (i + 2) jobs in a row -> first sample
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

        // FIXME: d must not be empty
        assert!(!d.is_empty());
        CurvePrefix { min_distance: d }
    }

    fn extrapolate_next(&self) -> Duration {
        let n = self.min_distance.len();
        assert!(n >= 2);
        // we are using n - k - 1 here because we don't store n=0 and n=1, so the
        // index is offset by 2
        (0..=(n / 2))
            .map(|k| self.min_distance[k] + self.min_distance[n - k - 1])
            .max()
            .unwrap()
    }

    pub fn extrapolate(&mut self, horizon: Duration) {
        // we cannot meaningfully extrapolate degenerate cases, so let's skip those
        if self.min_distance.len() >= 2 {
            while self.largest_known_distance() < horizon {
                self.min_distance.push(self.extrapolate_next())
            }
        }
    }

    pub fn extrapolate_steps(&mut self, n: usize) {
        // we cannot meaningfully extrapolate degenerate cases, so let's skip those
        if self.min_distance.len() >= 2 {
            while self.jobs_in_largest_known_distance() < n {
                self.min_distance.push(self.extrapolate_next())
            }
        }
    }

    fn min_job_separation(&self) -> Duration {
        // minimum separation of two jobs given by first element
        self.min_distance[0]
    }

    fn largest_known_distance(&self) -> Duration {
        *self.min_distance.last().unwrap()
    }

    fn jobs_in_largest_known_distance(&self) -> usize {
        self.min_distance.len()
    }

    // note: does not extrapolate
    fn lookup_arrivals(&self, delta: Duration) -> usize {
        // TODO: for really large vectors, this should be a binary search...
        for (i, distance_of_njobs) in self.min_distance.iter().enumerate() {
            let njobs = i + 2; // we do not store n=0 and n=1
            if delta <= *distance_of_njobs {
                return njobs - 1;
            }
        }
        // should never get here
        panic!()
    }

    // note: does not extrapolate, so extremely pessimistic for n > self.min_distance.len() - 1
    pub fn min_distance(&self, n: usize) -> Duration {
        if n > 1 {
            // account for the fact that we store distances only for 2+ jobs
            self.min_distance[(n - 2).min(self.min_distance.len() - 1)]
        } else {
            0
        }
    }

    fn jitter_shift(&mut self, jitter: Duration) {
        for d in self.min_distance.iter_mut() {
            // shorten minimum distance by the added jitter
            *d = if *d > jitter { *d - jitter } else { 0 };
       }
    }
}

impl FromIterator<Duration> for CurvePrefix {
    fn from_iter<I: IntoIterator<Item = Duration>>(iter: I) -> CurvePrefix {
        let mut distances: Vec<Duration> = iter.into_iter().collect();
        // ensure the min-distance function is monotonic
        for i in 1..distances.len() {
            distances[i] = distances[i].max(distances[i - 1]);
        }
        assert!(!distances.is_empty());
        CurvePrefix {
            min_distance: distances,
        }
    }
}

impl ArrivalBound for CurvePrefix {
    fn number_arrivals(&self, delta: Duration) -> usize {
        if delta > 0 {
            // first, resolve long delta by super-additivity of arrival curves
            let prefix = delta / self.largest_known_distance();
            let prefix_jobs = prefix as usize * self.jobs_in_largest_known_distance();
            let tail = delta % self.largest_known_distance();
            if tail > self.min_job_separation() {
                prefix_jobs + self.lookup_arrivals(tail) as usize
            } else {
                prefix_jobs + (tail > 0) as usize
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
            idx: usize,
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

        Box::new(StepsIter {
            sum: 1,
            step_sizes: diffs,
            idx: 0,
        })
    }

    fn clone_with_jitter(&self, jitter: Duration) -> Box<dyn ArrivalBound> {
        let mut ab = Box::new(self.clone());
        ab.jitter_shift(jitter);
        ab
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

pub struct ExtrapolatingCurvePrefix {
    prefix: RefCell<CurvePrefix>,
    jitter: Duration,
}

impl ExtrapolatingCurvePrefix {
    pub fn new(curve: CurvePrefix) -> Self {
        ExtrapolatingCurvePrefix {
            prefix: RefCell::new(curve),
            jitter: 0
        }
    }
}

impl ArrivalBound for ExtrapolatingCurvePrefix {
    fn number_arrivals(&self, delta: Duration) -> usize {
        if delta == 0 {
            // special case: delta=0 always yields 0
            0
        } else {
            // extrapolate up to the requested duration
            let mut curve = self.prefix.borrow_mut();
            curve.extrapolate(self.jitter + delta + 1);
            curve.number_arrivals(self.jitter + delta)
        }
    }

    fn steps_iter<'a>(&'a self) -> Box<dyn Iterator<Item = Duration> + 'a> {
        struct StepsIter<'a> {
            dist: Instant,
            curve: &'a ExtrapolatingCurvePrefix,
            njobs: usize,
        }

        impl<'a> StepsIter<'a> {
            fn advance(&mut self) {
                let mut prefix = self.curve.prefix.borrow_mut();
                while prefix.min_distance(self.njobs) <= self.dist + self.curve.jitter {
                    prefix.extrapolate_steps(self.njobs + 1);
                    self.njobs += 1
                }
                self.dist = prefix.min_distance(self.njobs) - self.curve.jitter;
            }
        }

        impl<'a> Iterator for StepsIter<'a> {
            type Item = Duration;

            fn next(&mut self) -> Option<Self::Item> {
                let val = 1 + self.dist;
                self.advance();
                Some(val)
            }
        }

        Box::new(StepsIter { dist: 0, curve: self, njobs: 0 })
    }

    fn clone_with_jitter(&self, jitter: Duration) -> Box<dyn ArrivalBound> {
        Box::new(ExtrapolatingCurvePrefix{
            prefix: RefCell::new(self.prefix.borrow().clone()),
            jitter: self.jitter + jitter,
        })
    }
}


#[derive(Copy, Clone, Debug)]
pub struct Poisson {
    pub rate: f64,
}

impl Poisson {
    pub fn arrival_probability(&self, delta: Duration, njobs: usize) -> f64 {
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

#[derive(Copy, Clone, Debug)]
pub struct ApproximatedPoisson {
    poisson: Poisson,
    epsilon: f64,
}

impl ArrivalBound for ApproximatedPoisson {
    fn number_arrivals(&self, delta: Duration) -> usize {
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
            njobs
        } else {
            0
        }
    }

    fn clone_with_jitter(&self, jitter: Duration) -> Box<dyn ArrivalBound> {
        Box::new(Propagated {
            response_time_jitter: jitter,
            input_event_model: self.clone(),
        })
    }
}

pub struct Propagated<T: ArrivalBound> {
    pub response_time_jitter: Duration,
    pub input_event_model: T,
}

impl<T: ArrivalBound> ArrivalBound for Propagated<T> {
    fn number_arrivals(&self, delta: Duration) -> usize {
        if delta > 0 {
            self.input_event_model
                .number_arrivals(delta + self.response_time_jitter)
        } else {
            0
        }
    }

    fn steps_iter<'a>(&'a self) -> Box<dyn Iterator<Item = Duration> + 'a> {
        Box::new(
            iter::once(1).chain(
                // shift the steps of the input event model earlier by the jitter amount
                self.input_event_model
                    .steps_iter()
                    .filter(move |x| *x > self.response_time_jitter)
                    .map(move |x| x - self.response_time_jitter),
            ),
        )
    }

    fn clone_with_jitter(&self, added_jitter: Duration) -> Box<dyn ArrivalBound> {
        Box::new(Propagated {
            response_time_jitter: self.response_time_jitter + added_jitter,
            // a bit of hack to get around liftime trouble
            input_event_model: self.input_event_model.clone_with_jitter(0),
        })
    }
}

impl<T: ArrivalBound> ArrivalBound for [T] {
    fn number_arrivals(&self, delta: Duration) -> usize {
        self.iter().map(|ab| ab.number_arrivals(delta)).sum()
    }

    fn steps_iter<'a>(&'a self) -> Box<dyn Iterator<Item = Duration> + 'a> {
        Box::new(self.iter().map(|ab| ab.steps_iter()).kmerge().dedup())
    }

    fn clone_with_jitter(&self, jitter: Duration) -> Box<dyn ArrivalBound> {
        let cloned: Vec<Box<dyn ArrivalBound>> =
            self.iter().map(|ab| ab.clone_with_jitter(jitter)).collect();
        Box::new(cloned)
    }
}

// repeated implementation for Vec<T> because otherwise Vec<Box<dyn ArrivalBound>>
// is not recognized as an ArrivalBound, despite the above blanket implementation for
impl<T: ArrivalBound> ArrivalBound for Vec<T> {
    fn number_arrivals(&self, delta: Duration) -> usize {
        self.iter().map(|ab| ab.number_arrivals(delta)).sum()
    }

    fn steps_iter<'a>(&'a self) -> Box<dyn Iterator<Item = Duration> + 'a> {
        Box::new(self.iter().map(|ab| ab.steps_iter()).kmerge().dedup())
    }

    fn clone_with_jitter(&self, jitter: Duration) -> Box<dyn ArrivalBound> {
        let cloned: Vec<Box<dyn ArrivalBound>> =
            self.iter().map(|ab| ab.clone_with_jitter(jitter)).collect();
        Box::new(cloned)
    }
}
