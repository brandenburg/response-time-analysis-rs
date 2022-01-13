pub mod arrival;
pub mod demand;
pub mod fifo;
pub mod fixed_point;
pub mod fixed_priority;
pub mod ros2;
pub mod supply;
pub mod time;
pub mod wcet;

#[cfg(test)]
mod tests {
    use crate::time::{Duration, Service};

    // helper function for typed duration values
    pub fn d(val: u64) -> Duration {
        Duration::from(val)
    }

    // helper function for vectors of typed duration values
    pub fn dv(vals: &[u64]) -> Vec<Duration> {
        vals.iter().map(|t| d(*t)).collect()
    }

    // helper function for typed service values
    pub fn s(val: u64) -> Service {
        Service::from(val)
    }

    // helper function for vectors of typed service values
    pub fn sv(vals: &[u64]) -> Vec<Service> {
        vals.iter().map(|t| s(*t)).collect()
    }
}
