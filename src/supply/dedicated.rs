use super::SupplyBound;
use crate::time::{Duration, Service};

/// A trivial model to represent a 100%-available, dedicated processor.
///
/// There are no delays due to resource unavailability under this model.
#[derive(Debug, Clone, Copy)]
pub struct Dedicated {
    // nothing to define here
}

impl Dedicated {
    /// Construct a new dedicated processor dummy.
    pub fn new() -> Dedicated {
        Dedicated {}
    }
}

impl SupplyBound for Dedicated {
    fn provided_service(&self, delta: Duration) -> Service {
        Service::from(delta)
    }

    fn service_time(&self, demand: Service) -> Duration {
        Duration::from(demand)
    }
}
