# Response-Time Analysis (RTA) in Rust

This crate provides interfaces, definitions, and algorithms for the response-time analysis of real-time systems.

## Scope: A Toolkit, not a Tool

The crate does *not* provide a ready-made tool itself, is not specific to any particular input format or set of assumptions, and does not offer or require a canonical task representation. Rather, it is intended as a low-level library of reusable definitions and analyses, held together by a bunch of traits. Based on this foundation, higher-level facilities (and one-off research tools) may be built.

## Attribution

Some of the analyses provided in this crate stem from the following publications:

- K. Bedarkar, M. Vardishvili, S. Bozhko, M. Maida, and B. Brandenburg, “[From Intuition to Coq: A Case Study in Verified Response-Time Analysis of FIFO Scheduling](https://people.mpi-sws.org/~bbb/papers/pdf/rtss22.pdf)”, *Proceedings of the 43rd IEEE Real-Time Systems Symposium (RTSS 2022)*, pp.&nbsp;197–210, December 2022.
- T. Blaß, D. Casini, S. Bozhko, and  B. Brandenburg, “[A ROS 2 Response-Time Analysis Exploiting Starvation Freedom and Execution-Time Variance](https://people.mpi-sws.org/~bbb/papers/pdf/rtss21-ros.pdf)”, *Proceedings of the 42nd IEEE Real-Time Systems Symposium (RTSS 2021)*, pp.&nbsp;41--53, December 2021.  
- S. Bozhko and B. Brandenburg, “[Abstract Response-Time Analysis: A Formal Foundation for the Busy-Window Principle](https://drops.dagstuhl.de/opus/volltexte/2020/12385/pdf/LIPIcs-ECRTS-2020-22.pdf)”,  *Proceedings of the 32nd Euromicro Conference on Real-Time Systems (ECRTS 2020)*, pp.&nbsp;22:1--22:24, July 2020.  
- D. Casini, T. Blaß, I. Lütkebohle, and B. Brandenburg, “[Response-Time Analysis of ROS&nbsp;2 Processing Chains under Reservation-Based Scheduling](https://people.mpi-sws.org/~bbb/papers/pdf/ecrts19-rev1.pdf)”, *Proceedings of the 31st Euromicro Conference on Real-Time Systems (ECRTS 2019)*, pp.&nbsp;6:1--6:23, July 2019.  

When using analyses from this crate for academic work, please cite the corresponding papers.

## Contributions

Patches and feedback are welcome. Please open issues and/or pull requests on GitHub.

## Maintainer

Please contact [Björn Brandenburg](https://www.mpi-sws.org/~bbb) in case of questions. 