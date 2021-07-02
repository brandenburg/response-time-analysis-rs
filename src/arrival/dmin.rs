use std::iter;

use super::ArrivalBound;
use crate::time::Duration;

struct DeltaMinIterator<'a, AB: ArrivalBound> {
    ab: &'a AB,
    steps: Box<dyn Iterator<Item = Duration> + 'a>,
    step_count: usize,
    next_step: Option<Duration>,
    next_count: usize,
}

impl<'a, AB: ArrivalBound> DeltaMinIterator<'a, AB> {
    fn new(ab: &'a AB) -> Self {
        DeltaMinIterator {
            ab,
            steps: ab.steps_iter(),
            next_step: None,
            next_count: 2,
            step_count: 0,
        }
    }

    fn advance(&mut self) {
        while self.step_count < self.next_count {
            self.next_step = self.steps.next();
            if let Some(delta) = self.next_step {
                self.step_count = self.ab.number_arrivals(delta);
            } else {
                break;
            }
        }
    }
}

impl<'a, AB: ArrivalBound> Iterator for DeltaMinIterator<'a, AB> {
    type Item = (usize, Duration);

    fn next(&mut self) -> Option<Self::Item> {
        self.advance();
        if let Some(delta) = self.next_step {
            let dmin = Some((self.next_count, delta - Duration::from(1)));
            self.next_count += 1;
            dmin
        } else {
            None
        }
    }
}

pub fn nonzero_delta_min_iter<'a>(
    ab: &'a impl ArrivalBound,
) -> impl Iterator<Item = (usize, Duration)> + 'a {
    // don't both bother with the two default cases (zero and one jobs)
    DeltaMinIterator::new(ab)
}

pub fn delta_min_iter<'a>(
    ab: &'a impl ArrivalBound,
) -> impl Iterator<Item = (usize, Duration)> + 'a {
    // first the two default cases for zero and one jobs
    iter::once((0, Duration::from(0)))
        .chain(iter::once((1, Duration::from(0))))
        .chain(DeltaMinIterator::new(ab))
}
