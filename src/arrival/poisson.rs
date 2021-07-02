use super::{ArrivalBound, Propagated};
use crate::time::{Duration, Time};

/// Model of a [Poisson](https://en.wikipedia.org/wiki/Poisson_distribution) arrival process.
#[derive(Copy, Clone, Debug)]
pub struct Poisson {
    /// Mean arrival rate (lambda).
    pub rate: f64,
}

impl Poisson {
    pub fn arrival_probability(&self, delta: Duration, njobs: usize) -> f64 {
        // quick and dirty naive factorial: k!
        let mut denominator = 1.0;
        for x in 1..(njobs + 1) {
            denominator *= x as f64;
        }
        let mean = Time::from(delta) as f64 * self.rate;
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

/// A finite approximation of a Poisson process with bounded
/// probability of under-approximation.
///
/// A Poisson arrival process as modeled by [Poisson] cannot comply
/// with the the [ArrivalBound] interface as there is a diminishing,
/// but non-zero probability for an arbitrary large number of
/// arrivals to occur in any interval. The  [ApproximatedPoisson]
/// process implements the [ArrivalBound] interface with a
/// configurable residual probability of excessive arrivals.
///
/// The bound [number_arrivals][ApproximatedPoisson::number_arrivals]
/// ensures that the claimed number of job arrivals are not exceeded
/// with probability at least `1 - epsilon`.
///
/// Note: the implementation is intended to be simple, but not
/// necessarily fast, so it's best to convert it into an [arrival
/// curve][super::Curve] before use if performance is of interest.
#[derive(Copy, Clone, Debug)]
pub struct ApproximatedPoisson {
    /// The underlying Poisson process.
    poisson: Poisson,
    /// The acceptable probability of under-approximating the number
    /// of arrivals. Must be small but non-zero.
    epsilon: f64,
}

impl ArrivalBound for ApproximatedPoisson {
    /// Bound the number of jobs released in any interval of length `delta` with probability `1 - epsilon`.
    fn number_arrivals(&self, delta: Duration) -> usize {
        if delta.is_non_zero() {
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
        Box::new(Propagated::with_jitter(self, jitter))
    }
}
