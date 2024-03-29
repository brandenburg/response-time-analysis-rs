use std::cell::RefCell;
use std::collections::VecDeque;
use std::iter::{self, FromIterator};
use std::rc::Rc;

use super::{
    divide_with_ceil, nonzero_delta_min_iter, ArrivalBound, Periodic, Propagated, Sporadic,
};
use crate::time::{Duration, Offset};

/// An arrival curve (also commonly called an "upper event arrival
/// curve" *η+*) that can describe arbitrarily bursty sporadic
/// arrival processes.
///
/// As is common, the arrival curve is defined by a finite *delta-min
/// vector* that describes the minimum interval length in which a
/// certain number of jobs may arrive.
#[derive(Clone, Debug)]
pub struct Curve {
    min_distance: Vec<Duration>,
}

impl Curve {
    /// Construct a new arrival curve.
    ///
    /// The `delta_min_prefix` is the *delta-min prefix* describing
    /// the arrival curve. Each element of the vector gives the
    /// minimum interval length in which a corresponding number of
    /// jobs may arrive.

    /// **Convention**: we do not store the mininum distance for 0
    /// and 1 jobs. So the `min_distance` vector at offset 0 contains
    /// the minimum separation of two jobs, the `min_distance` vector
    /// at offset 1 contains the length of the shortest interval in
    /// which three jobs arrive, and so on.
    pub fn new(delta_min_prefix: Vec<Duration>) -> Curve {
        assert!(!delta_min_prefix.is_empty());
        Curve {
            min_distance: delta_min_prefix,
        }
    }

    /// Obtain an arrival curve by inferring a delta-min vector from
    /// any given arrival process `T`.
    ///
    /// The delta-min vector is chosen such that it covers all
    /// arrivals until the given `horizon`.
    pub fn from_arrival_bound_until<T: ArrivalBound>(ab: &T, horizon: Duration) -> Curve {
        Self::new(
            nonzero_delta_min_iter(&ab)
                .enumerate()
                .take_while(|(count, (_njobs, delta))| *delta <= horizon || *count < 2)
                .map(|(_count, (_njobs, delta))| delta)
                .collect(),
        )
    }

    /// Obtain an arrival curve by inferring a delta-min vector from
    /// any given arrival process `T`.
    ///
    /// The delta-min vector is chosen such that it covers at least
    /// `up_to_njobs` job arrivals.
    pub fn from_arrival_bound<T: ArrivalBound>(ab: &T, up_to_njobs: usize) -> Curve {
        Self::new(
            nonzero_delta_min_iter(&ab)
                .enumerate()
                .take_while(|(count, (njobs, _delta))| *njobs <= up_to_njobs || *count < 2)
                .map(|(_count, (_njobs, delta))| delta)
                .collect(),
        )
    }

    /// Obtain an arrival curve by inferring a delta-min prefix from
    /// a given trace of arrival events, expressed as the offset of
    /// the event from a reference time zero.
    ///
    /// The resultant delta-min vector will consist of `prefix_jobs`
    /// entries (if there are a sufficient number of arrivals in the
    /// trace).
    pub fn from_trace(arrival_times: impl Iterator<Item = Offset>, prefix_jobs: usize) -> Curve {
        let mut d: Vec<Duration> = Vec::with_capacity(prefix_jobs);
        let mut window: VecDeque<Offset> = VecDeque::with_capacity(prefix_jobs + 1);

        // consider all job arrivals in the trace
        for t in arrival_times {
            // sanity check: the arrival times must be monotonic
            assert!(t >= *(window.back().unwrap_or(&t)));
            // look at all arrival times in the sliding window, in order
            // from most recent to oldest
            for (i, v) in window.iter().rev().enumerate() {
                // Compute the separation from the current arrival t to the arrival
                // of the (i + 1)-th preceding job.
                // So if i=0, we are looking at two adjacent jobs.
                let observed_gap = v.distance_to(t);
                if d.len() <= i {
                    // we have not yet seen (i + 2) jobs in a row -> first sample
                    d.push(observed_gap)
                } else {
                    // update belief if we have seen two events with
                    // less separation than previously observed
                    d[i] = d[i].min(observed_gap)
                }
            }
            // add arrival time to sliding window
            window.push_back(t);
            // trim sliding window if necessary
            if window.len() > prefix_jobs {
                window.pop_front();
            }
        }

        // FIXME: d must not be empty
        Curve::new(d)
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

    fn can_extrapolate(&self) -> bool {
        // we cannot meaningfully extrapolate degenerate cases, so let's skip those
        self.min_distance.len() >= 2
    }

    /// Extend the underlying delta-min prefix by exploiting the
    /// [subadditivity](https://en.wikipedia.org/wiki/Subadditivity)
    /// of proper arrival curves until the delta-min prefix covers
    /// intervals of length `horizon`.
    pub fn extrapolate(&mut self, horizon: Duration) {
        if self.can_extrapolate() {
            while self.largest_known_distance() < horizon {
                self.min_distance.push(self.extrapolate_next())
            }
        }
    }

    /// Extend the underlying delta-min prefix by exploiting the
    /// [subadditivity](https://en.wikipedia.org/wiki/Subadditivity)
    /// of proper arrival curves by `n` elements.
    pub fn extrapolate_steps(&mut self, n: usize) {
        if self.can_extrapolate() {
            while self.jobs_in_largest_known_distance() < n {
                self.min_distance.push(self.extrapolate_next())
            }
        }
    }

    /// Extend the underlying delta-min prefix by one element while
    /// exploiting _both_ the
    /// [subadditivity](https://en.wikipedia.org/wiki/Subadditivity)
    /// of proper arrival curves _and_ the provided bound on the next
    /// step. The bound is a tuple (`delta`, `njobs`), where `njobs`
    /// *must* be the element that the underlying delta-min vector is
    /// extrapolated to.
    pub fn extrapolate_with_bound(&mut self, bound: (Duration, usize)) {
        let (delta, njobs) = bound;
        // We subtract epsilon since we store the distance
        // between jobs, not the interval length.
        let dmin = delta - Duration::epsilon();
        // check that we've been given the expected upper bound
        // (+ 2 because we don't store the values for 0 and 1 jobs)
        if self.min_distance.len() + 2 == njobs {
            if self.can_extrapolate() {
                let extrapolated = self.extrapolate_next();
                self.min_distance.push(dmin.max(extrapolated))
            } else {
                // If we cannot extrapolate, simply take the given bound.
                self.min_distance.push(dmin)
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

    /// Return a lower bound on the length of an interval in which
    /// `n` arrival events occur.
    ///
    /// Note: does not extrapolate, so extremely pessimistic if `n`
    /// exceeds the length of the internal minimum-distance prefix.
    pub fn min_distance(&self, n: usize) -> Duration {
        if n > 1 {
            // account for the fact that we store distances only for 2+ jobs
            self.min_distance[(n - 2).min(self.min_distance.len() - 1)]
        } else {
            Duration::zero()
        }
    }
}

impl FromIterator<Duration> for Curve {
    fn from_iter<I: IntoIterator<Item = Duration>>(iter: I) -> Curve {
        let mut distances: Vec<Duration> = iter.into_iter().collect();
        // ensure the min-distance function is monotonic
        for i in 1..distances.len() {
            distances[i] = distances[i].max(distances[i - 1]);
        }
        assert!(!distances.is_empty());
        Curve {
            min_distance: distances,
        }
    }
}

impl ArrivalBound for Curve {
    fn number_arrivals(&self, delta: Duration) -> usize {
        if delta.is_non_zero() {
            // first, resolve long delta by super-additivity of arrival curves
            let prefix = delta / self.largest_known_distance();
            let prefix_jobs = prefix as usize * self.jobs_in_largest_known_distance();
            let tail = delta % self.largest_known_distance();
            if tail > self.min_job_separation() {
                prefix_jobs + self.lookup_arrivals(tail) as usize
            } else {
                prefix_jobs + tail.is_non_zero() as usize
            }
        } else {
            0
        }
    }

    fn steps_iter<'a>(&'a self) -> Box<dyn Iterator<Item = Duration> + 'a> {
        let diffs: Vec<_> = iter::once(Duration::zero())
            .chain(self.min_distance.iter().copied())
            .zip(self.min_distance.iter().copied())
            .map(|(a, b)| b - a)
            .filter(|d| d.is_non_zero())
            .collect();

        struct StepsIter {
            sum: Duration,
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
            sum: Duration::from(1),
            step_sizes: diffs,
            idx: 0,
        })
    }

    fn clone_with_jitter(&self, jitter: Duration) -> Box<dyn ArrivalBound> {
        Box::new(Propagated::with_jitter(self, jitter))
    }
}

impl From<Periodic> for Curve {
    fn from(p: Periodic) -> Self {
        Curve {
            min_distance: vec![p.period],
        }
    }
}

impl From<Sporadic> for Curve {
    fn from(s: Sporadic) -> Self {
        let jitter_jobs = divide_with_ceil(s.jitter, s.min_inter_arrival);
        // Jitter can cause pessimism when applying super-additivity.
        // Memory is cheap. Hence, unroll quite a bit to avoid running into
        // pessimism too frequently.
        // By default, unroll until the jitter jobs are no more than 10% of the
        // jobs of the jobs in the unrolled interval, and until for at least 500 jobs.
        let n = 500.max(jitter_jobs as usize * 10);
        Curve::from_arrival_bound(&s, n)
    }
}

/// An arrival curve that automatically extrapolates and
/// caches extrapolation results using interior mutability.
#[derive(Clone)]
pub struct ExtrapolatingCurve {
    prefix: Rc<RefCell<Curve>>,
}

impl ExtrapolatingCurve {
    /// Construct a new auto-extrapolating arrival curve by wrapping
    /// a given non-extrapolating curve.
    pub fn new(curve: Curve) -> Self {
        ExtrapolatingCurve {
            prefix: Rc::new(RefCell::new(curve)),
        }
    }
}

impl ArrivalBound for ExtrapolatingCurve {
    fn number_arrivals(&self, delta: Duration) -> usize {
        if delta.is_zero() {
            // special case: delta=0 always yields 0
            0
        } else {
            // extrapolate up to the requested duration
            let mut curve = self.prefix.borrow_mut();
            curve.extrapolate(delta + Duration::from(1));
            curve.number_arrivals(delta)
        }
    }

    fn steps_iter<'a>(&'a self) -> Box<dyn Iterator<Item = Duration> + 'a> {
        struct StepsIter<'a> {
            dist: Duration,
            curve: &'a ExtrapolatingCurve,
            njobs: usize,
        }

        impl<'a> StepsIter<'a> {
            fn advance(&mut self) {
                let mut prefix = self.curve.prefix.borrow_mut();
                while prefix.min_distance(self.njobs) <= self.dist {
                    prefix.extrapolate_steps(self.njobs + 1);
                    self.njobs += 1
                }
                self.dist = prefix.min_distance(self.njobs);
            }
        }

        impl<'a> Iterator for StepsIter<'a> {
            type Item = Duration;

            fn next(&mut self) -> Option<Self::Item> {
                let val = Duration::from(1) + self.dist;
                self.advance();
                Some(val)
            }
        }

        let prefix = self.prefix.borrow();
        if prefix.can_extrapolate() {
            Box::new(StepsIter {
                dist: Duration::zero(),
                curve: self,
                njobs: 0,
            })
        } else {
            // degenerate case --- don't have info to extrapolate,
            // so just return the periodic process implied by the single-value
            // dmin function
            let period = prefix.min_distance(2);
            Box::new((0..).map(move |j| period * j + Duration::from(1)))
        }
    }

    fn clone_with_jitter(&self, jitter: Duration) -> Box<dyn ArrivalBound> {
        Box::new(Propagated::with_jitter(self, jitter))
    }
}
