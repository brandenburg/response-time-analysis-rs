/*! Models of arrival processes (periodic, sporadic, etc.)

This module provides a central trait, [ArrivalBound], which represents
an upper-bounding arrival curve. Furthermore, it provides
implementations of the trait for several types of arrival processes
commonly studied in the literature on the analysis of real-time systems
(e.g, [Periodic] and [Sporadic] tasks).
*/

use auto_impl::auto_impl;
use itertools::Itertools;

use crate::time::Duration;

/// The main interface for models describing arrival processes.
#[auto_impl(&, Box, Rc)]
pub trait ArrivalBound {
    /// Bound the number of jobs released in any interval of length `delta`.
    fn number_arrivals(&self, delta: Duration) -> usize;

    /// Yield the sequence of interval lengths (i.e., values of `delta` in
    /// [ArrivalBound::number_arrivals]) for which the arrival bound
    /// "steps", i.e., where it shows an increase in the number of
    /// released jobs.
    ///
    /// More precisely, the iterator yields values of `delta` such that:
    ///
    /// `self.number_arrivals(delta - 1) < self.number_arrivals(delta)`.
    ///
    /// Defaults to using [ArrivalBound::brute_force_steps_iter],
    /// which is very slow, so implementors should override this method.
    fn steps_iter<'a>(&'a self) -> Box<dyn Iterator<Item = Duration> + 'a> {
        self.brute_force_steps_iter()
    }

    /// Same semantics as [ArrivalBound::steps_iter], but provided by
    /// a default implementation in the most naive way possible.
    /// Avoid if performance is at all important.
    fn brute_force_steps_iter<'a>(&'a self) -> Box<dyn Iterator<Item = Duration> + 'a> {
        let (a1, a2) = (0..)
            .map(move |delta| (delta, self.number_arrivals(Duration::from(delta))))
            .tee();
        Box::new(
            a1.zip(a2.skip(1))
                .filter(|((_, njobs1), (_, njobs2))| njobs1 != njobs2)
                .map(|((_, _), (d2, _))| Duration::from(d2)),
        )
    }

    /// Clone the arrival model while accounting for added release
    /// jitter. Returns a boxed `dyn` object because the underlying
    /// type may change.
    fn clone_with_jitter(&self, jitter: Duration) -> Box<dyn ArrivalBound>;
}

mod aggregated;
mod arrival_curve_prefix;
mod curve;
mod dmin;
mod never;
mod periodic;
mod poisson;
mod propagated;
mod slice;
mod sporadic;

pub use aggregated::sum_of;
pub use arrival_curve_prefix::ArrivalCurvePrefix;
pub use curve::{Curve, ExtrapolatingCurve};
pub use dmin::{delta_min_iter, nonzero_delta_min_iter};
pub use never::Never;
pub use periodic::Periodic;
pub use poisson::{ApproximatedPoisson, Poisson};
pub use propagated::Propagated;
pub use sporadic::Sporadic;

// common helper function
fn divide_with_ceil(a: Duration, b: Duration) -> u64 {
    a / b + (a % b > Duration::from(0)) as u64
}

#[cfg(test)]
mod tests;
