/*! An RTA for *first-in first-out* (**FIFO**) scheduling

This module provides an RTA for the FIFO policy, also commonly known as
*first-come first-serve* (FCFS) scheduling.

## Citation

The provided analysis was proposed in the following paper:

- K. Bedarkar, M. Vardishvili, S. Bozhko, M. Maida, and B. Brandenburg, “[From Intuition to Coq: A Case Study in Verified Response-Time Analysis of FIFO Scheduling](https://people.mpi-sws.org/~bbb/papers/pdf/rtss22.pdf)”, *Proceedings of the 43rd IEEE Real-Time Systems Symposium (RTSS 2022)*, pp.&nbsp;197–210, December 2022.

Please cite the paper when using this module for academic work.

 */

mod rta;
pub use rta::dedicated_uniproc_rta;

#[cfg(test)]
mod tests;
