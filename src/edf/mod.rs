/*! RTAs for *earliest-deadline first* (**EDF**) scheduling

This module collects RTAs for the EDF policy.

## Citation

The provided analyses are based on the following paper:

- S. Bozhko and B. Brandenburg, “[Abstract Response-Time Analysis: A Formal Foundation for the Busy-Window Principle](https://drops.dagstuhl.de/opus/volltexte/2020/12385/pdf/LIPIcs-ECRTS-2020-22.pdf)”,  *Proceedings of the 32nd Euromicro Conference on Real-Time Systems (ECRTS 2020)*, pp.&nbsp;22:1--22:24, July 2020.

Please cite the paper when using functionality from this module for academic work.

 */

pub mod floating_nonpreemptive;
pub mod fully_nonpreemptive;
pub mod fully_preemptive;
pub mod limited_preemptive;

#[cfg(test)]
mod tests;
