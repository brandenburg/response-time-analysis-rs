use std::iter;

use super::ArrivalBound;
use crate::time::Duration;

/// Pathological corner case: model of a task that never releases any jobs.
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
        Box::new(Never {})
    }
}
