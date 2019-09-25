use crate::time::Duration;

pub trait SupplyBound {
    fn provided_service(&self, delta: Duration) -> Duration;
}

pub struct DedicatedProcessor {
    // nothing to define here
}

impl SupplyBound for DedicatedProcessor {
    fn provided_service(&self, delta: Duration) -> Duration {
        delta
    }
}


pub struct Periodic {
    pub period: Duration,
    pub budget: Duration
}

impl SupplyBound for Periodic {
    fn provided_service(&self, delta: Duration) -> Duration {
        // Supply bound function of the periodic resource model, 
        // as given by Shin & Lee (RTSS 2003).

        let slack = dbg!(self.period - self.budget);
        if slack > delta {
            return 0
        }
        // implicit floor due to integer division
        let full_periods = dbg!((delta - slack) / self.period);
        let x = dbg!(slack + slack + full_periods * self.period);
        let fractional_period = if dbg!(x < delta) { dbg!(delta - x) } else { 0 };

        full_periods * self.budget + fractional_period
    }
}


