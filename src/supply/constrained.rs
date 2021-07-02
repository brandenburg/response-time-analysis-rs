use super::SupplyBound;
use crate::time::{Duration, Service};

/// A periodic resource with reduced jitter.
///
/// This model represents a refined variant of of the
/// [Periodic][super::Periodic] supply model wherein the availability
/// of budget within each period is restricted by a constrained
/// relative deadline.
///
/// The client(s) of this supply is/are guaranteed (at least)
/// `budget` time units of processor service every `period` time
/// units within `deadline` time units w.r.t. the start of the
/// period.
#[derive(Debug, Clone, Copy)]
pub struct Constrained {
    pub period: Duration,
    pub budget: Service,
    pub deadline: Duration,
}

impl Constrained {
    /// Construct a new constrained periodic resource model such that
    /// `budget <= deadline <= period`.
    pub fn new(budget: Service, deadline: Duration, period: Duration) -> Self {
        assert!(Duration::from(budget) <= deadline);
        assert!(deadline <= period);
        Constrained {
            period,
            budget,
            deadline,
        }
    }
}

impl SupplyBound for Constrained {
    fn provided_service(&self, delta: Duration) -> Service {
        let budget = Duration::from(self.budget);
        let shift = self.period - budget;
        if shift > delta {
            return Service::none();
        }
        // implicit floor due to integer division
        let full_periods = (delta - shift) / self.period;
        let x = shift + self.period * full_periods + self.deadline - budget;
        let fractional_period = if x < delta {
            self.budget.min(Service::from(delta - x))
        } else {
            Service::none()
        };

        self.budget * full_periods + fractional_period
    }

    fn service_time(&self, demand: Service) -> Duration {
        if demand.is_none() {
            return Duration::zero();
        }

        let budget = Duration::from(self.budget);
        let demand = Duration::from(demand);

        // implicit floor due to integer division
        let full_periods = demand / budget;
        let full_budget = budget * full_periods;
        let fractional_budget = if full_budget < demand {
            demand - full_budget + self.period - budget
        } else {
            Duration::zero()
        };

        self.deadline - budget + self.period * full_periods + fractional_budget
    }
}
