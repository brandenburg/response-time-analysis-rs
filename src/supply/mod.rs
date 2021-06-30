use auto_impl::auto_impl;

use crate::time::{Duration, Service};

/// Generic interface for models of processor supply.
#[auto_impl(&, Box, Rc)]
pub trait SupplyBound {
    /// Bound the minimum amount of service provided during an
    /// interval of length `delta`.
    fn provided_service(&self, delta: Duration) -> Service;

    /// Bound the maximum interval length during which the supply
    /// provides at least `demand` amount of service.
    fn service_time(&self, demand: Service) -> Duration {
        let mut t = demand;
        loop {
            let supply = self.provided_service(t);
            if supply >= demand {
                return t;
            }
            // jump ahead by how much is still missing
            t += demand - supply;
        }
    }
}

mod constrained;
mod dedicated;
mod periodic;

pub use constrained::Constrained;
pub use dedicated::Dedicated;
pub use periodic::Periodic;