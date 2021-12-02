use super::{ArrivalBound, Curve, Propagated};
use crate::time::Duration;
use std::iter;

/// Representation of a step of an arrival curve within a finite prefix.
type Step = (Duration, usize);

/// An alternate representation of an arbitrary arrival curve intended
/// primarily for input purposes.
///
/// Whereas the [Curve] arrival bound is based on a delta-min
/// representation, this variant expresses the underlying arrival
/// curve by means of a finite sequence of `steps` and a `horizon`.
/// This makes this representation easier to use in cases where an
/// arrival curve is not already given in delta-min representation.
///
/// Up to the `horizon`, this representation produces exact results.
/// Beyond the horizon, it uses a quick, but potentially quite
/// pessimistic extrapolation. For better extrapolation, convert the
/// [ArrivalCurvePrefix] to a [Curve] (via [std::convert::From]) and
/// use its extrapolation facilities (or simply wrap it as an
/// [ExtrapolatingCurve](super::ExtrapolatingCurve)).
#[derive(Clone, Debug)]
pub struct ArrivalCurvePrefix {
    horizon: Duration,
    steps: Vec<Step>,
}

// A step-based implementation of an eta-max curve (i.e., arrival curve).
impl ArrivalCurvePrefix {
    /// Construct a new `ArrivalCurvePrefix` from a given `horizon`
    /// and a sequence of `steps`.
    ///
    /// The given sequence of `steps` is a list of tuples of the form
    /// `(delta, n)`, with the meaning that the arrival curve first assumes
    /// the value `n` for the interval length `delta`.
    ///
    /// The given sequence of `steps` must be contained in the
    /// horizon and strictly monotonic in `n`.
    pub fn new(horizon: Duration, steps: Vec<Step>) -> ArrivalCurvePrefix {
        // make sure the prefix is well-formed
        let mut last_njobs: usize = 0;
        for (delta, njobs) in &steps {
            assert!(*delta <= horizon);
            assert!(last_njobs < *njobs);
            last_njobs = *njobs;
        }
        ArrivalCurvePrefix { horizon, steps }
    }

    fn max_njobs_in_horizon(&self) -> usize {
        self.steps.last().map(|(_, njobs)| *njobs).unwrap_or(0)
    }

    fn lookup(&self, delta: Duration) -> usize {
        assert!(delta <= self.horizon);
        if delta.is_zero() {
            0
        } else {
            let step = self
                .steps
                .iter()
                .enumerate()
                .skip_while(|(_i, (min_distance, _njobs))| *min_distance <= delta)
                .map(|(i, _)| i)
                .next();
            let i = step.unwrap_or(self.steps.len());
            self.steps[i - 1].1
        }
    }
}

impl ArrivalBound for ArrivalCurvePrefix {
    fn number_arrivals(&self, delta: Duration) -> usize {
        let full_horizons = delta / self.horizon;
        let partial_horizon = delta % self.horizon;
        self.max_njobs_in_horizon() * (full_horizons as usize) + self.lookup(partial_horizon)
    }

    fn clone_with_jitter(&self, jitter: Duration) -> Box<dyn ArrivalBound> {
        Box::new(Propagated::with_jitter(self, jitter))
    }

    fn steps_iter<'a>(&'a self) -> Box<dyn Iterator<Item = Duration> + 'a> {
        let horizon = self.horizon;
        Box::new(
            iter::once(Duration::zero()).chain((0..).flat_map(move |cycle: u64| {
                self.steps
                    .iter()
                    .map(move |(offset, _njobs)| *offset + horizon * cycle)
            })),
        )
    }
}

impl From<&ArrivalCurvePrefix> for Curve {
    fn from(ac: &ArrivalCurvePrefix) -> Self {
        let njobs = ac.max_njobs_in_horizon();
        // construct equivalent curve from jobs in horizon
        let mut curve = Curve::from_arrival_bound(&ac, njobs);
        // trigger extrapolation to one additional job to account for horizon
        curve.extrapolate_with_bound((ac.horizon + Duration::epsilon(), njobs + 1));
        curve
    }
}

impl From<ArrivalCurvePrefix> for Curve {
    fn from(ac: ArrivalCurvePrefix) -> Self {
        Self::from(&ac)
    }
}
