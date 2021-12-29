use crate::tests::{s, sv};
use crate::wcet::{self, JobCostModel};

use std::iter::FromIterator;

#[test]
fn cost_models() {
    let wcet: wcet::Scalar = wcet::Scalar::from(s(10));

    assert_eq!(wcet.cost_of_jobs(0), s(0));
    assert_eq!(wcet.cost_of_jobs(3), s(30));
    assert_eq!(wcet.cost_of_jobs(10), s(100));
    let jobs1: Vec<_> = wcet.job_cost_iter().take(5).collect();
    assert_eq!(jobs1, sv(&[10, 10, 10, 10, 10]));

    let multi_frame: wcet::Multiframe = wcet::Multiframe::new(vec![s(3), s(2), s(1)]);
    assert_eq!(multi_frame.cost_of_jobs(0), s(0));
    assert_eq!(multi_frame.cost_of_jobs(1), s(3));
    assert_eq!(multi_frame.cost_of_jobs(2), s(5));
    assert_eq!(multi_frame.cost_of_jobs(3), s(6));
    assert_eq!(multi_frame.cost_of_jobs(4), s(9));
    assert_eq!(multi_frame.cost_of_jobs(5), s(11));
    assert_eq!(multi_frame.cost_of_jobs(6), s(12));
    let jobs2: Vec<_> = multi_frame.job_cost_iter().take(5).collect();
    assert_eq!(jobs2, sv(&[3, 2, 1, 3, 2]));

    let trace = vec![1, 1, 3, 1, 2, 2, 1, 3, 1, 0, 0, 3, 2, 0, 1, 1];
    let cf = wcet::Curve::from_trace(trace.iter().map(|t| s(*t)), 3);
    assert_eq!(cf.cost_of_jobs(0), s(0));
    assert_eq!(cf.cost_of_jobs(1), s(3));
    assert_eq!(cf.cost_of_jobs(2), s(5));
    assert_eq!(cf.cost_of_jobs(3), s(6));
    assert_eq!(cf.cost_of_jobs(4), s(9));
    assert_eq!(cf.cost_of_jobs(5), s(11));
    assert_eq!(cf.cost_of_jobs(6), s(12));
    let jobs3: Vec<_> = cf.job_cost_iter().take(10).collect();
    assert_eq!(jobs3, sv(&[3, 2, 1, 3, 2, 1, 3, 2, 1, 3]));

    let wcets = vec![15, 25, 30];
    let cf2 = wcet::Curve::from_iter(wcets.iter().map(|t| s(*t)));
    assert_eq!(cf2.cost_of_jobs(0), s(0));
    assert_eq!(cf2.cost_of_jobs(1), s(15));
    assert_eq!(cf2.cost_of_jobs(2), s(25));
    assert_eq!(cf2.cost_of_jobs(3), s(30));
    assert_eq!(cf2.cost_of_jobs(4), s(45));
    assert_eq!(cf2.cost_of_jobs(5), s(55));
    assert_eq!(cf2.cost_of_jobs(6), s(60));
    let jobs4: Vec<_> = cf2.job_cost_iter().take(10).collect();
    assert_eq!(jobs4, sv(&[15, 10, 5, 15, 10, 5, 15, 10, 5, 15]));
}

#[test]
fn cost_extrapolation() {
    let wcets = vec![100, 101, 102, 103, 104, 105, 205, 206];
    let mut cf = wcet::Curve::from_iter(wcets.iter().map(|t| s(*t)));
    assert_eq!(cf.cost_of_jobs(9), s(306));
    cf.extrapolate(10);
    assert_eq!(cf.cost_of_jobs(9), s(207));

    let wcets2 = vec![145, 149, 151, 153, 157, 160, 163, 166, 168, 171, 174];
    let cf2 = wcet::ExtrapolatingCurve::new(wcet::Curve::from_iter(wcets2.iter().map(|t| s(*t))));
    assert_eq!(cf2.cost_of_jobs(11), s(174));
    assert_eq!(cf2.cost_of_jobs(12), s(319));
    assert_eq!(cf2.cost_of_jobs(4), s(153));
    assert_eq!(cf2.cost_of_jobs(9), s(168));
    assert_eq!(cf2.cost_of_jobs(4 + 9), s(153 + 168));
}
