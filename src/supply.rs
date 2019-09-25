use crate::time::Duration;

pub trait SupplyBound {
    fn provided_service(&self, delta: Duration) -> Duration;

    fn service_time(&self, demand: Duration) -> Duration {
        let mut t = demand;
        loop {
            let supply = self.provided_service(t);
            if supply >= demand {
                return t
            }
            // jump ahead by how much is still missing
            t += demand - supply;
        }
    }
}

pub struct DedicatedProcessor {
    // nothing to define here
}

impl SupplyBound for DedicatedProcessor {
    fn provided_service(&self, delta: Duration) -> Duration {
        delta
    }
    
    fn service_time(&self, demand: Duration) -> Duration {
        demand
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

        let slack = self.period - self.budget;
        if slack > delta {
            return 0
        }
        // implicit floor due to integer division
        let full_periods = (delta - slack) / self.period;
        let x = slack + slack + full_periods * self.period;
        let fractional_period = if x < delta { delta - x } else { 0 };

        full_periods * self.budget + fractional_period
    }

    fn service_time(&self, demand: Duration) -> Duration {
        let slack = self.period - self.budget;

        if demand == 0 {
            return 0
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


