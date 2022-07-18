/*! An RTA for *first-in first-out* (**FIFO**) scheduling

This module provides an RTA for the FIFO policy, also commonly known as
*first-come first-serve* (FCFS) scheduling.
 */

mod rta;
pub use rta::dedicated_uniproc_rta;

#[cfg(test)]
mod tests;
