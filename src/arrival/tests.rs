use super::{ArrivalBound, ArrivalCurvePrefix, Curve, Periodic, Sporadic};
use crate::time::Duration;

fn d(val: u64) -> Duration {
    Duration::from(val)
}

#[test]
fn arrival_curve_prefix_steps_iter() {
    let horizon = d(100);
    let steps = vec![(d(1), 1), (d(10), 2), (d(21), 3), (d(45), 4)];
    let ac = ArrivalCurvePrefix::new(horizon, steps);

    let mut steps = ac.steps_iter();

    assert_eq!(steps.next(), Some(d(0)));
    assert_eq!(steps.next(), Some(d(1)));
    assert_eq!(steps.next(), Some(d(10)));
    assert_eq!(steps.next(), Some(d(21)));
    assert_eq!(steps.next(), Some(d(45)));
    assert_eq!(steps.next(), Some(d(101)));
    assert_eq!(steps.next(), Some(d(110)));
    assert_eq!(steps.next(), Some(d(121)));
    assert_eq!(steps.next(), Some(d(145)));
    assert_eq!(steps.next(), Some(d(201)));
    assert_eq!(steps.next(), Some(d(210)));
    assert_eq!(steps.next(), Some(d(221)));
    assert_eq!(steps.next(), Some(d(245)));
}

#[test]
#[allow(clippy::identity_op)]
fn arrival_curve_prefix_number_arrivals() {
    let horizon = d(100);
    let steps = vec![(d(1), 1), (d(10), 2), (d(21), 3), (d(45), 4)];
    let ac = ArrivalCurvePrefix::new(horizon, steps);

    assert_eq!(ac.number_arrivals(d(0)), 0);
    assert_eq!(ac.number_arrivals(d(5)), 1);
    assert_eq!(ac.number_arrivals(d(9)), 1);
    assert_eq!(ac.number_arrivals(d(10)), 2);
    assert_eq!(ac.number_arrivals(d(15)), 2);
    assert_eq!(ac.number_arrivals(d(24)), 3);
    assert_eq!(ac.number_arrivals(d(42)), 3);
    assert_eq!(ac.number_arrivals(d(57)), 4);

    assert_eq!(ac.number_arrivals(d(100)), 0 + 4);
    assert_eq!(ac.number_arrivals(d(105)), 1 + 4);
    assert_eq!(ac.number_arrivals(d(109)), 1 + 4);
    assert_eq!(ac.number_arrivals(d(110)), 2 + 4);
    assert_eq!(ac.number_arrivals(d(115)), 2 + 4);
    assert_eq!(ac.number_arrivals(d(124)), 3 + 4);
    assert_eq!(ac.number_arrivals(d(142)), 3 + 4);
    assert_eq!(ac.number_arrivals(d(157)), 4 + 4);

    assert_eq!(ac.number_arrivals(d(200)), 0 + 8);
    assert_eq!(ac.number_arrivals(d(205)), 1 + 8);
    assert_eq!(ac.number_arrivals(d(209)), 1 + 8);
    assert_eq!(ac.number_arrivals(d(210)), 2 + 8);
    assert_eq!(ac.number_arrivals(d(215)), 2 + 8);
    assert_eq!(ac.number_arrivals(d(224)), 3 + 8);
    assert_eq!(ac.number_arrivals(d(242)), 3 + 8);
    assert_eq!(ac.number_arrivals(d(257)), 4 + 8);
}

#[test]
fn arrival_curve_prefix_to_curve_1() {
    let horizon = d(5000);
    let steps = vec![(d(1), 1)];
    let arr_curve = ArrivalCurvePrefix::new(horizon, steps);
    let curve = Curve::from(arr_curve);

    assert_eq!(curve.min_distance(2), d(5000));
}

#[test]
fn arrival_curve_prefix_to_curve_2() {
    let horizon = d(220);
    let steps = vec![(d(1), 1), (d(105), 2)];
    let arr_curve = ArrivalCurvePrefix::new(horizon, steps);
    let curve = Curve::from(arr_curve);

    assert_eq!(curve.min_distance(2), d(104));
    assert_eq!(curve.min_distance(3), d(220));
}

#[test]
fn arrival_curve_prefix_to_curve_3() {
    let horizon = d(11);
    let steps = vec![(d(1), 1), (d(3), 2), (d(5), 3), (d(7), 4)];
    let arr_curve = ArrivalCurvePrefix::new(horizon, steps);
    let curve = Curve::from(arr_curve);

    assert_eq!(curve.min_distance(2), d(2));
    assert_eq!(curve.min_distance(3), d(4));
    assert_eq!(curve.min_distance(4), d(6));
    assert_eq!(curve.min_distance(5), d(11));
}

#[test]
fn arrival_curve_prefix_to_curve_4() {
    let horizon = d(4);
    let steps = vec![(d(1), 2), (d(2), 3), (d(4), 5)];
    let arr_curve = ArrivalCurvePrefix::new(horizon, steps);
    let curve = Curve::from(arr_curve);

    assert_eq!(curve.min_distance(2), d(0));
    assert_eq!(curve.min_distance(3), d(1));
    assert_eq!(curve.min_distance(4), d(3));
    assert_eq!(curve.min_distance(5), d(3));
    assert_eq!(curve.min_distance(6), d(4));
}

#[test]
fn arrival_curve_prefix_to_curve_dmin_constrained() {
    let horizon = d(10);
    let steps = vec![(d(1), 1), (d(5), 2), (d(9), 3)];
    let acp = ArrivalCurvePrefix::new(horizon, steps);
    let ac = Curve::from(&acp);

    // the horizon is pessimistic
    assert_eq!(acp.number_arrivals(d(11)), 4);
    // the curve extrapolated correctly
    assert_eq!(ac.number_arrivals(d(11)), 3);
}

#[test]
fn arrival_curve_prefix_to_curve_horizon_constrained() {
    let horizon = d(100);
    let steps = vec![(d(1), 1), (d(5), 2), (d(9), 3)];
    let acp = ArrivalCurvePrefix::new(horizon, steps);
    let ac = Curve::from(&acp);

    // the horizon is more constraining than the prefix
    assert_eq!(acp.number_arrivals(d(50)), 3);
    // the curve reflects this information
    assert_eq!(ac.number_arrivals(d(50)), 3);
}

#[test]
fn curve_from_periodic() {
    let p = Periodic::new(d(15));
    let s = Sporadic::from(p);
    let cp = Curve::from_arrival_bound(&p, 12);
    let cs = Curve::unroll_sporadic(&s, d(155));

    for delta in 0..=150 {
        let delta = d(delta);
        assert_eq!(p.number_arrivals(delta), cp.number_arrivals(delta));
        assert_eq!(p.number_arrivals(delta), cs.number_arrivals(delta));
    }
}
