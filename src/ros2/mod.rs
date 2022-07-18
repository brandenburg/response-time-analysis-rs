/*! RTAs for the default **ROS2** executor

This module collects RTAs for the default executor of the ROS2 framework.

## Citations

The analyses provided in this module are based on the following publications:

- T. Blaß, D. Casini, S. Bozhko, and  B. Brandenburg, “[A ROS 2 Response-Time Analysis Exploiting Starvation Freedom and Execution-Time Variance](https://people.mpi-sws.org/~bbb/papers/pdf/rtss21-ros.pdf)”, *Proceedings of the 42nd IEEE Real-Time Systems Symposium (RTSS 2021)*, pp.&nbsp;41--53, December 2021.
- D. Casini, T. Blaß, I. Lütkebohle, and B. Brandenburg, “[Response-Time Analysis of ROS&nbsp;2 Processing Chains under Reservation-Based Scheduling](https://people.mpi-sws.org/~bbb/papers/pdf/ecrts19-rev1.pdf)”, *Proceedings of the 31st Euromicro Conference on Real-Time Systems (ECRTS 2019)*, pp.&nbsp;6:1--6:23, July 2019.

Please cite these papers when using functionality from this module.

 */

mod ecrts19;

/// Busy-window-aware analysis that exploits both the non-starvation
/// property of the ROS2 callback scheduler and the busy-window
/// principle, due to [Blaß et al. (2021)](https://people.mpi-sws.org/~bbb/papers/pdf/rtss21-ros.pdf).
pub mod bw;
/// Round-robin-aware analysis that exploits the non-starvation
/// property of the ROS2 callback scheduler, due to [Blaß et al. (2021)](https://people.mpi-sws.org/~bbb/papers/pdf/rtss21-ros.pdf).
pub mod rr;

pub use ecrts19::{rta_event_source, rta_polling_point_callback, rta_processing_chain, rta_timer};

#[cfg(test)]
mod tests;
