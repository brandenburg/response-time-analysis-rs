use super::SupplyBound;
use crate::time::Duration;

/// A trivial model to represent a 100%-available, dedicated processor.
///
/// There are no delays due to resource unavailability under this model.
#[derive(Debug, Clone, Copy)]
pub struct Dedicated {
    // nothing to define here
}

impl SupplyBound for Dedicated {
    fn provided_service(&self, delta: Duration) -> Duration {
        delta
    }

    fn service_time(&self, demand: Duration) -> Duration {
        demand
    }
}
