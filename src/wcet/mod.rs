/*! Models of resource needs such as a task's scalar *worst-case execution time* (WCET)

This module provides the trait [JobCostModel], which models the notion
of the cumulative worst-case resource needs of one or more jobs of a
task.

The implementation most commonly encountered in the literature is
[Scalar], which represents a WCET value as assumed by Liu & Layland. */

use auto_impl::auto_impl;

use crate::time::Service;

/// The interface for models of per-job and per-job-sequence
/// *maximum* execution costs.
#[auto_impl(&, Box, Rc)]
pub trait JobCostModel {
    /// Model: yield the maximum cumulative processor demand of any
    /// `n` consecutive jobs.
    fn cost_of_jobs(&self, n: usize) -> Service {
        self.job_cost_iter().take(n).sum()
    }

    /// Model: yield the WCET of the job with the least WCET among
    /// any sequence of `n` consecutive jobs.
    fn least_wcet(&self, n: usize) -> Service {
        self.job_cost_iter()
            .take(n)
            .min()
            .unwrap_or_else(Service::none)
    }

    /// Model: iterate the maximum WCETs of any sequence of consecutive jobs.
    fn job_cost_iter<'a>(&'a self) -> Box<dyn Iterator<Item = Service> + 'a>;
}

mod curve;
mod multiframe;
mod scalar;

pub use curve::{Curve, ExtrapolatingCurve};
pub use multiframe::Multiframe;
pub use scalar::Scalar;

#[cfg(test)]
mod tests;
