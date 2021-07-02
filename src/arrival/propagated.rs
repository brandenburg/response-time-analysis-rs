use std::iter;

use super::ArrivalBound;
use crate::time::Duration;

/// A simple model of arrivals induced by a precedence relationship.
///
/// Suppose two components *A* and *B* are connected such that each
/// activation of *A* subsequently triggers (up to) one activation of
/// *B* (e.g., *A* is a producer and *B* a consumer). Then, if
/// `input_event_model` is the arrival model of *A* and activations
/// of *A* have a maximum response time bounded by
/// `response_time_jitter`, then this arrival model upper-bounds the
/// activations of *B*.
pub struct Propagated<T: ArrivalBound> {
    pub response_time_jitter: Duration,
    pub input_event_model: T,
}

impl<T: ArrivalBound + Clone> Propagated<T> {
    pub fn with_jitter(event_model: &T, response_time_jitter: Duration) -> Self {
        Propagated {
            input_event_model: event_model.clone(),
            response_time_jitter,
        }
    }
}

impl<T: ArrivalBound + Clone + 'static> ArrivalBound for Propagated<T> {
    fn number_arrivals(&self, delta: Duration) -> usize {
        if delta.is_non_zero() {
            self.input_event_model
                .number_arrivals(delta + self.response_time_jitter)
        } else {
            0
        }
    }

    fn steps_iter<'a>(&'a self) -> Box<dyn Iterator<Item = Duration> + 'a> {
        Box::new(
            iter::once(Duration::from(1)).chain(
                // shift the steps of the input event model earlier by the jitter amount
                self.input_event_model
                    .steps_iter()
                    .filter(move |x| *x > self.response_time_jitter + Duration::from(1))
                    .map(move |x| x - self.response_time_jitter),
            ),
        )
    }

    fn clone_with_jitter(&self, added_jitter: Duration) -> Box<dyn ArrivalBound> {
        Box::new(Propagated {
            response_time_jitter: self.response_time_jitter + added_jitter,
            input_event_model: self.input_event_model.clone(),
        })
    }
}
