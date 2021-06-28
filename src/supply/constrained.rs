use super::SupplyBound;
use crate::time::Duration;

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
    pub budget: Duration,
    pub deadline: Duration,
}

impl Constrained {
    /// Construct a new constrained periodic resource model such that
    /// `budget <= deadline <= period`.
    pub fn new(budget: Duration, deadline: Duration, period: Duration) -> Self {
        assert!(budget <= deadline);
        assert!(deadline <= period);
        Constrained {
            period,
            budget,
            deadline,
        }
    }
}

impl SupplyBound for Constrained {
    fn provided_service(&self, delta: Duration) -> Duration {
        let shift = self.period - self.budget;
        if shift > delta {
            return 0;
        }
        // implicit floor due to integer division
        let full_periods = (delta - shift) / self.period;
        let x = shift + full_periods * self.period + self.deadline - self.budget;
        let fractional_period = if x < delta {
            self.budget.min(delta - x)
        } else {
            0
        };

        full_periods * self.budget + fractional_period
    }

    fn service_time(&self, demand: Duration) -> Duration {
        if demand == 0 {
            return 0;
        }

        // implicit floor due to integer division
        let full_periods = demand / self.budget;
        let full_budget = full_periods * self.budget;
        let fractional_budget = if full_budget < demand {
            demand - full_budget + self.period - self.budget
        } else {
            0
        };

        self.deadline - self.budget + full_periods * self.period + fractional_budget
    }
}
