use itertools::Itertools;

use super::ArrivalBound;
use crate::time::Duration;

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

#[derive(Clone, Debug)]
struct SumOf<AB1: ArrivalBound, AB2: ArrivalBound>(AB1, AB2);

/// The sum of two arrival bounds, representing the joint arrival bound.
pub fn sum_of<AB1: ArrivalBound, AB2: ArrivalBound>(ab1: AB1, ab2: AB2) -> impl ArrivalBound {
    SumOf(ab1, ab2)
}

impl<AB1: ArrivalBound, AB2: ArrivalBound> ArrivalBound for SumOf<AB1, AB2> {
    fn number_arrivals(&self, delta: Duration) -> usize {
        self.0.number_arrivals(delta) + self.1.number_arrivals(delta)
    }

    fn steps_iter<'a>(&'a self) -> Box<dyn Iterator<Item = Duration> + 'a> {
        Box::new(self.0.steps_iter().merge(self.1.steps_iter()).dedup())
    }

    fn clone_with_jitter(&self, jitter: Duration) -> Box<dyn ArrivalBound> {
        Box::new(SumOf(
            self.0.clone_with_jitter(jitter),
            self.1.clone_with_jitter(jitter),
        ))
    }
}
