/*!
# Response-Time Analysis (RTA)

This crate collects interfaces, definitions, and algorithms for the
response-time analysis of real-time systems.

## Scope

The crate does *not* provide a ready-made tool itself, is not specific
to any particular input format or set of assumptions, and does not offer
or require a canonical task representation. Rather, it is intended as a
low-level library of reusable definitions and analyses, held together by
a bunch of traits.  Based on this foundation, higher-level facilities
(and one-off research tools) may be built.

## Citations

Some of the algorithms and analyses implemented in this crate come from
published papers, as mentioned in the documentation. If you use this
crate for academic work, please cite the paper(s) corresponding to the
analysis that you are using.



*/

pub mod arrival;
pub mod demand;
pub mod edf;
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
