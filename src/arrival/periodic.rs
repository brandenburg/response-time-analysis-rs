use super::{divide_with_ceil, ArrivalBound, Sporadic};
use crate::time::Duration;

/// Classic jitter-free periodic arrival process as introduced by Liu & Layland.
#[derive(Copy, Clone, Debug)]
pub struct Periodic {
    /// The exact separation between two job releases.
    pub period: Duration,
}

impl Periodic {
    /// Construct a new periodic arrival model.
    pub fn new(period: Duration) -> Periodic {
        Periodic { period }
    }
}

impl ArrivalBound for Periodic {
    fn number_arrivals(&self, delta: Duration) -> usize {
        divide_with_ceil(delta, self.period) as usize
    }

    fn steps_iter<'a>(&'a self) -> Box<dyn Iterator<Item = Duration> + 'a> {
        Box::new((0..).map(move |j| self.period * j + Duration::from(1)))
    }

    fn clone_with_jitter(&self, jitter: Duration) -> Box<dyn ArrivalBound> {
        let mut ab = Box::new(Sporadic::from(*self));
        ab.jitter = jitter;
        ab
    }
}
