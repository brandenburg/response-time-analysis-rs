use super::SupplyBound;
use crate::time::{Duration, Service};

/// The classic periodic resource model.
///
/// The client(s) of this supply is/are guaranteed (at least)
/// `budget` time units of processor service every `period` time
/// units.
#[derive(Debug, Clone, Copy)]
pub struct Periodic {
    pub period: Duration,
    pub budget: Service,
}

impl Periodic {
    /// Construct a new periodic supply, where `budget <= period`.
    pub fn new(budget: Service, period: Duration) -> Self {
        assert!(Duration::from(budget) <= period);
        Periodic { period, budget }
    }
}

impl SupplyBound for Periodic {
    fn provided_service(&self, delta: Duration) -> Service {
        // Supply bound function of the periodic resource model,
        // as given by Shin & Lee (RTSS 2003).

        let budget = Duration::from(self.budget);

        let slack = self.period - budget;
        if slack > delta {
            return Service::none();
        }
        // implicit floor due to integer division
        let full_periods = (delta - slack) / self.period;
        let x = slack + slack + self.period * full_periods;
        let fractional_period = if x < delta {
            Service::from(delta - x)
        } else {
            Service::none()
        };

        self.budget * full_periods + fractional_period
    }

    fn service_time(&self, demand: Service) -> Duration {
        if demand.is_none() {
            return Duration::zero();
        }

        let demand = Duration::from(demand);
        let budget = Duration::from(self.budget);
        let slack = self.period - budget;

        // implicit floor due to integer division
        let full_periods = demand / budget;
        let full_budget = budget * full_periods;
        let fractional_budget = if full_budget < demand {
            slack + demand - full_budget
        } else {
            Duration::zero()
        };

        slack + self.period * full_periods + fractional_budget
    }
}
