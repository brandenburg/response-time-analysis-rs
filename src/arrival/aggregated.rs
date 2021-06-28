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
