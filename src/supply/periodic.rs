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
        assert!(budget <= period);
        Periodic { period, budget }
    }
}

impl SupplyBound for Periodic {
    fn provided_service(&self, delta: Duration) -> Service {
        // Supply bound function of the periodic resource model,
        // as given by Shin & Lee (RTSS 2003).

        let slack = self.period - self.budget;
        if slack > delta {
            return 0;
        }
        // implicit floor due to integer division
        let full_periods = (delta - slack) / self.period;
        let x = slack + slack + full_periods * self.period;
        let fractional_period = if x < delta { delta - x } else { 0 };

        full_periods * self.budget + fractional_period
    }

    fn service_time(&self, demand: Service) -> Duration {
        let slack = self.period - self.budget;

        if demand == 0 {
            return 0;
        }

        // implicit floor due to integer division
        let full_periods = demand / self.budget;
        let full_budget = full_periods * self.budget;
        let fractional_budget = if full_budget < demand {
            slack + demand - full_budget
        } else {
            0
        };

        slack + self.period * full_periods + fractional_budget
    }
}
