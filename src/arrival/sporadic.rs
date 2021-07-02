use std::iter;

use super::{divide_with_ceil, ArrivalBound, Periodic};
use crate::time::Duration;

/// The classic sporadic arrival model (originally due to Mok) with release jitter.
///
/// A note on terminology: following standard convention, the
/// *arrival time* of a job denotes the time at which (conceptually)
/// the job is triggered, whereas its *release time* is the time at
/// which it actually becomes ready for execution.
#[derive(Copy, Clone, Debug)]
pub struct Sporadic {
    /// The minimum inter-arrival separation between any two job
    /// *arrivals* of the task.
    pub min_inter_arrival: Duration,
    /// The maximum release jitter, i.e., the maximum time between
    /// the *arrival* and the *release* of a job.
    pub jitter: Duration,
}

impl ArrivalBound for Sporadic {
    fn number_arrivals(&self, delta: Duration) -> usize {
        if delta.is_non_zero() {
            divide_with_ceil(delta + self.jitter, self.min_inter_arrival) as usize
        } else {
            0
        }
    }

    fn steps_iter<'a>(&'a self) -> Box<dyn Iterator<Item = Duration> + 'a> {
        Box::new(
            iter::once(Duration::from(1)).chain(
                (1..)
                    .filter(move |j| self.min_inter_arrival * *j + Duration::from(1) > self.jitter)
                    .map(move |j| self.min_inter_arrival * j + Duration::from(1) - self.jitter),
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
            jitter: Duration::zero(),
        }
    }
}
