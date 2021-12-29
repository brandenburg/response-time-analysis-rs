use assert_approx_eq::assert_approx_eq;
use std::iter::FromIterator;

use crate::arrival::{self, ArrivalBound, ArrivalCurvePrefix, Curve, Periodic, Sporadic};
use crate::time::{Duration, Offset};

use crate::tests::{d, dv};

fn brute_force_iter_check<T: ArrivalBound>(ab: &T) {
    let si100 = ab.steps_iter().take(100);
    let bf100 = ab.brute_force_steps_iter().take(100);

    for (s1, s2) in si100.zip(bf100) {
        assert_eq!(s1, s2)
    }
}

#[test]
fn periodic_arrivals() {
    let a = arrival::Periodic { period: d(10) };
    assert_eq!(a.number_arrivals(d(0)), 0);
    assert_eq!(a.number_arrivals(d(1)), 1);
    assert_eq!(a.number_arrivals(d(8)), 1);
    assert_eq!(a.number_arrivals(d(10)), 1);
    assert_eq!(a.number_arrivals(d(11)), 2);
    assert_eq!(a.number_arrivals(d(12)), 2);
    assert_eq!(a.number_arrivals(d(13)), 2);
    assert_eq!(a.number_arrivals(d(100)), 10);
    assert_eq!(a.number_arrivals(d(105)), 11);
}

#[test]
fn period_arrivals_dmin() {
    let ab = arrival::Periodic { period: d(10) };
    let dmin_ref = vec![
        (0, d(0)),
        (1, d(0)),
        (2, d(10)),
        (3, d(20)),
        (4, d(30)),
        (5, d(40)),
    ];
    let dmin = arrival::delta_min_iter(&ab);

    for (should, is) in dmin_ref.iter().zip(dmin) {
        assert_eq!(*should, is);
    }
}

#[test]
fn periodic_arrivals_via_unroll_sporadic() {
    let p = arrival::Periodic { period: d(10) };
    let s = arrival::Sporadic::from(p);
    let a = arrival::Curve::unroll_sporadic(&s, d(1000));
    assert_eq!(a.number_arrivals(d(0)), 0);
    assert_eq!(a.number_arrivals(d(1)), 1);
    assert_eq!(a.number_arrivals(d(8)), 1);
    assert_eq!(a.number_arrivals(d(10)), 1);
    assert_eq!(a.number_arrivals(d(11)), 2);
    assert_eq!(a.number_arrivals(d(12)), 2);
    assert_eq!(a.number_arrivals(d(13)), 2);
    assert_eq!(a.number_arrivals(d(100)), 10);
    assert_eq!(a.number_arrivals(d(101)), 11);
    assert_eq!(a.number_arrivals(d(105)), 11);
}

#[test]
fn periodic_arrivals_via_sporadic() {
    let p = arrival::Periodic { period: d(10) };
    let s = arrival::Sporadic::from(p);
    let a = arrival::Curve::from(s);
    assert_eq!(a.number_arrivals(d(0)), 0);
    assert_eq!(a.number_arrivals(d(1)), 1);
    assert_eq!(a.number_arrivals(d(8)), 1);
    assert_eq!(a.number_arrivals(d(10)), 1);
    assert_eq!(a.number_arrivals(d(11)), 2);
    assert_eq!(a.number_arrivals(d(12)), 2);
    assert_eq!(a.number_arrivals(d(13)), 2);
    assert_eq!(a.number_arrivals(d(100)), 10);
    assert_eq!(a.number_arrivals(d(101)), 11);
    assert_eq!(a.number_arrivals(d(105)), 11);
}
#[test]
fn periodic_arrivals_unrolled() {
    let p = arrival::Periodic { period: d(10) };
    let a = arrival::Curve::from(p);
    assert_eq!(a.number_arrivals(d(0)), 0);
    assert_eq!(a.number_arrivals(d(1)), 1);
    assert_eq!(a.number_arrivals(d(8)), 1);
    assert_eq!(a.number_arrivals(d(10)), 1);
    assert_eq!(a.number_arrivals(d(11)), 2);
    assert_eq!(a.number_arrivals(d(12)), 2);
    assert_eq!(a.number_arrivals(d(13)), 2);
    assert_eq!(a.number_arrivals(d(100)), 10);
    assert_eq!(a.number_arrivals(d(101)), 11);
    assert_eq!(a.number_arrivals(d(105)), 11);
}

#[test]
fn periodic_arrivals_from_trace() {
    let trace: Vec<u64> = vec![0, 10, 20, 30, 40];
    let a = arrival::Curve::from_trace(trace.iter().map(|x| Offset::from(*x)), 10);
    assert_eq!(a.number_arrivals(d(0)), 0);
    assert_eq!(a.number_arrivals(d(1)), 1);
    assert_eq!(a.number_arrivals(d(8)), 1);
    assert_eq!(a.number_arrivals(d(10)), 1);
    assert_eq!(a.number_arrivals(d(11)), 2);
    assert_eq!(a.number_arrivals(d(12)), 2);
    assert_eq!(a.number_arrivals(d(13)), 2);
    assert_eq!(a.number_arrivals(d(100)), 10);
    assert_eq!(a.number_arrivals(d(101)), 11);
    assert_eq!(a.number_arrivals(d(105)), 11);
}

#[test]
fn compare_periodic_arrivals() {
    let p = arrival::Periodic { period: d(10) };
    let s = arrival::Sporadic::from(p);
    let a = arrival::Curve::from(p);
    let b = arrival::Curve::from(s);
    let trace: Vec<u64> = vec![0, 10, 20, 30, 40];
    let t = arrival::Curve::from_trace(trace.iter().map(|x| Offset::from(*x)), 2);
    for delta in 0..1000 {
        assert_eq!(a.number_arrivals(d(delta)), p.number_arrivals(d(delta)));
        assert_eq!(s.number_arrivals(d(delta)), p.number_arrivals(d(delta)));
        assert_eq!(a.number_arrivals(d(delta)), b.number_arrivals(d(delta)));
        assert_eq!(p.number_arrivals(d(delta)), t.number_arrivals(d(delta)));
    }
}

#[test]
fn periodic_iter() {
    let p = arrival::Periodic { period: d(10) };
    let steps: Vec<_> = p.steps_iter().take(5).collect();
    assert_eq!(steps, [d(1), d(11), d(21), d(31), d(41)]);
    brute_force_iter_check(&p);

    let p2 = arrival::Curve::from(arrival::Sporadic::from(p));
    let steps2: Vec<_> = p2.steps_iter().take(5).collect();
    assert_eq!(steps2, [d(1), d(11), d(21), d(31), d(41)]);
}

#[test]
fn sporadic_arrivals() {
    let a = arrival::Sporadic {
        min_inter_arrival: d(10),
        jitter: d(3),
    };
    assert_eq!(a.number_arrivals(d(0)), 0);
    assert_eq!(a.number_arrivals(d(1)), 1);
    assert_eq!(a.number_arrivals(d(8)), 2);
    assert_eq!(a.number_arrivals(d(10)), 2);
    assert_eq!(a.number_arrivals(d(11)), 2);
    assert_eq!(a.number_arrivals(d(100)), 11);
    assert_eq!(a.number_arrivals(d(107)), 11);
    assert_eq!(a.number_arrivals(d(108)), 12);
    assert_eq!(a.number_arrivals(d(1108)), 112);
}

#[test]
fn sporadic_arrivals_from_trace() {
    let trace: Vec<u64> = vec![0, 7, 17, 27, 37, 47, 57, 67, 77, 87, 110, 117];
    let a = arrival::Curve::from_trace(trace.iter().map(|x| Offset::from(*x)), 5);
    assert_eq!(a.number_arrivals(d(0)), 0);
    assert_eq!(a.number_arrivals(d(1)), 1);
    assert_eq!(a.number_arrivals(d(8)), 2);
    assert_eq!(a.number_arrivals(d(10)), 2);
    assert_eq!(a.number_arrivals(d(11)), 2);
}

#[test]
fn sporadic_arrivals_unrolled() {
    let s = arrival::Sporadic {
        min_inter_arrival: d(10),
        jitter: d(3),
    };
    let a = arrival::Curve::from(s);
    assert_eq!(a.number_arrivals(d(0)), 0);
    assert_eq!(a.number_arrivals(d(1)), 1);
    assert_eq!(a.number_arrivals(d(8)), 2);
    assert_eq!(a.number_arrivals(d(10)), 2);
    assert_eq!(a.number_arrivals(d(11)), 2);
    assert_eq!(a.number_arrivals(d(100)), 11);
    assert_eq!(a.number_arrivals(d(107)), 11);
    assert_eq!(a.number_arrivals(d(108)), 12);
    assert_eq!(a.number_arrivals(d(1108)), 112);
}

#[test]
fn sporadic_arrivals_large_jitter() {
    let a = arrival::Sporadic {
        min_inter_arrival: d(10),
        jitter: d(16),
    };
    assert_eq!(a.number_arrivals(d(0)), 0);
    assert_eq!(a.number_arrivals(d(1)), 2);
    assert_eq!(a.number_arrivals(d(4)), 2);
    assert_eq!(a.number_arrivals(d(5)), 3);
}

#[test]
fn compare_sporadic_arrivals() {
    let s = arrival::Sporadic {
        min_inter_arrival: d(10),
        jitter: d(16),
    };
    let a = arrival::Curve::from(s);
    for delta in 0..1000 {
        assert_eq!(a.number_arrivals(d(delta)), s.number_arrivals(d(delta)));
    }
}

#[test]
fn sporadic_iter() {
    let s1 = arrival::Sporadic {
        min_inter_arrival: d(10),
        jitter: d(3),
    };
    let steps1: Vec<_> = s1.steps_iter().take(6).collect();
    assert_eq!(steps1, dv(&[1, 8, 18, 28, 38, 48]));

    let s2 = arrival::Sporadic {
        min_inter_arrival: d(10),
        jitter: d(16),
    };
    let steps2: Vec<_> = s2.steps_iter().take(6).collect();
    assert_eq!(steps2, dv(&[1, 5, 15, 25, 35, 45]));

    let s3 = arrival::Curve::from(s1);
    let steps3: Vec<_> = s3.steps_iter().take(6).collect();
    assert_eq!(steps3, dv(&[1, 8, 18, 28, 38, 48]));

    let s4 = arrival::Curve::from(s2);
    let steps4: Vec<_> = s4.steps_iter().take(6).collect();
    assert_eq!(steps4, dv(&[1, 5, 15, 25, 35, 45]));

    brute_force_iter_check(&s1);
    brute_force_iter_check(&s2);
    brute_force_iter_check(&s3);
    brute_force_iter_check(&s4);
}

#[test]
fn poisson() {
    let p = arrival::Poisson { rate: 0.01 };
    assert_approx_eq!(p.arrival_probability(d(100), 0), 0.368, 0.001);
    assert_approx_eq!(p.arrival_probability(d(100), 1), 0.368, 0.001);
    assert_approx_eq!(p.arrival_probability(d(100), 2), 0.184, 0.001);
    assert_approx_eq!(p.arrival_probability(d(100), 3), 0.061, 0.001);
    assert_approx_eq!(p.arrival_probability(d(100), 4), 0.015, 0.001);
    assert_approx_eq!(p.arrival_probability(d(100), 5), 0.003, 0.001);
    assert_approx_eq!(p.arrival_probability(d(100), 6), 0.0005, 0.001);
}

#[test]
fn never_arrives() {
    let p = arrival::Never {};
    assert_eq!(p.number_arrivals(d(10)), 0);
    let steps: Vec<_> = p.steps_iter().collect();
    assert_eq!(steps, []);
    let prop = arrival::Propagated {
        input_event_model: p,
        response_time_jitter: d(3),
    };
    assert_eq!(prop.number_arrivals(d(10)), 0);
    let steps: Vec<_> = p.steps_iter().collect();
    assert_eq!(steps, []);
    let dmin: Vec<_> = arrival::nonzero_delta_min_iter(&p).collect();
    assert_eq!(dmin, []);
}

#[test]
fn propagated_jitter() {
    let p = arrival::Periodic { period: d(10) };
    let s = arrival::Sporadic {
        min_inter_arrival: d(10),
        jitter: d(3),
    };
    let prop = arrival::Propagated {
        input_event_model: p,
        response_time_jitter: d(3),
    };
    for t in 0..1000 {
        assert_eq!(s.number_arrivals(d(t)), prop.number_arrivals(d(t)));
    }
    for (x, y) in s.steps_iter().zip(prop.steps_iter().take(100)) {
        assert_eq!(x, y);
    }
    brute_force_iter_check(&p);
    brute_force_iter_check(&s);
}

#[test]
fn aggregated_arrivals() {
    let agg: Vec<Box<dyn ArrivalBound>> = vec![
        Box::new(arrival::Sporadic {
            min_inter_arrival: d(3),
            jitter: d(0),
        }),
        Box::new(arrival::Periodic { period: d(5) }),
    ];

    let ab = &agg;

    assert_eq!(ab.number_arrivals(d(0)), 0);
    assert_eq!(ab.number_arrivals(d(1)), 2);
    assert_eq!(ab.number_arrivals(d(2)), 2);
    assert_eq!(ab.number_arrivals(d(3)), 2);
    assert_eq!(ab.number_arrivals(d(4)), 3);
    assert_eq!(ab.number_arrivals(d(5)), 3);
    assert_eq!(ab.number_arrivals(d(6)), 4);
    assert_eq!(ab.number_arrivals(d(7)), 5);
    assert_eq!(ab.number_arrivals(d(8)), 5);
    assert_eq!(ab.number_arrivals(d(9)), 5);
    assert_eq!(ab.number_arrivals(d(10)), 6);
    assert_eq!(ab.number_arrivals(d(11)), 7);
    assert_eq!(ab.number_arrivals(d(12)), 7);
    assert_eq!(ab.number_arrivals(d(13)), 8);
    assert_eq!(ab.number_arrivals(d(14)), 8);
    assert_eq!(ab.number_arrivals(d(15)), 8);
    assert_eq!(ab.number_arrivals(d(16)), 10);

    let steps: Vec<Duration> = ab.steps_iter().take_while(|x| *x < d(17)).collect();
    assert_eq!(steps, dv(&[1, 4, 6, 7, 10, 11, 13, 16]));
    brute_force_iter_check(ab);

    let a = &ab[0..1];
    assert_eq!(a.number_arrivals(d(10)), 4);
    brute_force_iter_check(&a);

    let boxed = Box::new(agg);
    assert_eq!(boxed.number_arrivals(d(16)), 10);
    brute_force_iter_check(&boxed);

    let a_boxed = &boxed[0..1];
    assert_eq!(a_boxed.number_arrivals(d(10)), 4);

    let boxed_ptr: &dyn ArrivalBound = &boxed;
    assert_eq!(boxed_ptr.number_arrivals(d(10)), 6);
}

#[test]
fn curve_extrapolation() {
    let dmin: Vec<u64> = vec![1, 2, 12, 15, 18, 21];
    let mut curve = arrival::Curve::from_iter(dmin.iter().map(|t| d(*t)));

    curve.extrapolate(d(500));

    let dmin_ref: Vec<u64> = vec![
        0, 0, 1, 2, 12, 15, 18, 21, 27, 30, 33, 39, 42, 45, 51, 54, 57, 63, 66, 69, 75, 78, 81, 87,
        90, 93, 99, 102, 105, 111, 114, 117, 123, 126, 129, 135, 138, 141, 147, 150, 153, 159, 162,
        165, 171, 174, 177, 183, 186, 189, 195, 198, 201, 207, 210, 213, 219, 222, 225, 231, 234,
        237, 243, 246, 249, 255, 258, 261, 267, 270, 273, 279, 282, 285, 291, 294, 297, 303, 306,
        309, 315, 318, 321, 327, 330, 333, 339, 342, 345, 351, 354, 357, 363, 366, 369, 375, 378,
        381, 387, 390,
    ];

    for (x, dist) in dmin_ref.iter().enumerate() {
        if x > dmin.len() + 1 {
            let all_combinations: Vec<_> = (2..x)
                .map(|k| (k, curve.min_distance(x - k + 1) + curve.min_distance(k)))
                .collect();
            let y = all_combinations
                .iter()
                .map(|(_, y)| *y)
                .max()
                .unwrap_or(d(0));
            assert_eq!(y, curve.min_distance(x));
        }
        assert_eq!(d(*dist), curve.min_distance(x));
    }

    let ab_ref: Vec<usize> = vec![
        0, 1, 2, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 4, 4, 4, 5, 5, 5, 6, 6, 6, 7, 7, 7, 7, 7, 7, 8, 8,
        8, 9, 9, 9, 10, 10, 10, 10, 10, 10, 11, 11, 11, 12, 12, 12, 13, 13, 13, 13, 13, 13, 14, 14,
        14, 15, 15, 15, 16, 16, 16, 16, 16, 16, 17, 17, 17, 18, 18, 18, 19, 19, 19, 19, 19, 19, 20,
        20, 20, 21, 21, 21, 22, 22, 22, 22, 22, 22, 23, 23, 23, 24, 24, 24, 25, 25, 25, 25, 25, 25,
    ];

    for (delta, njobs) in ab_ref.iter().enumerate() {
        assert_eq!(*njobs, curve.number_arrivals(d(delta as u64)));
    }

    for (should, is) in (0..)
        .zip(dmin_ref.iter().map(|t| d(*t)))
        .zip(arrival::delta_min_iter(&curve))
    {
        assert_eq!(should, is);
    }
}

#[test]
fn curve_jitter_steps() {
    let dmin: Vec<u64> = vec![1, 2, 12, 15, 18, 21];
    let curve = arrival::Curve::from_iter(dmin.iter().map(|t| d(*t)));
    let curve_with_jitter = curve.clone_with_jitter(d(2));
    brute_force_iter_check(&curve);
    brute_force_iter_check(&curve_with_jitter);
}

#[test]
fn curve_on_demand_jitter() {
    let dmin: Vec<u64> = vec![1, 2, 12, 15, 18, 21];
    let mut curve = arrival::Curve::from_iter(dmin.iter().map(|t| d(*t)));
    let horizon = 1000;
    curve.extrapolate(d(horizon));
    let od_curve = arrival::ExtrapolatingCurve::new(curve.clone());

    let jitters: Vec<u64> = vec![2, 5, 10, 13, 17, 19, 21, 123];
    for j in jitters.iter() {
        let c1 = curve.clone_with_jitter(d(*j));
        let c2 = od_curve.clone_with_jitter(d(*j));

        for delta in 0..=(horizon - j) {
            assert_eq!(c1.number_arrivals(d(delta)), c2.number_arrivals(d(delta)))
        }

        for (s1, s2) in c1
            .steps_iter()
            .take_while(|s1| *s1 <= d(horizon - j))
            .zip(c2.steps_iter())
        {
            assert_eq!(s1, s2)
        }
    }

    let mut c1 = curve.clone_with_jitter(d(1));
    let mut c2 = od_curve.clone_with_jitter(d(1));
    let mut h = horizon - 1;

    for j in jitters.iter() {
        c1 = c1.clone_with_jitter(d(*j));
        c2 = c2.clone_with_jitter(d(*j));
        h -= *j;

        for delta in 0..=h {
            assert_eq!(c1.number_arrivals(d(delta)), c2.number_arrivals(d(delta)))
        }

        for (s1, s2) in c1
            .steps_iter()
            .take_while(|s1| *s1 <= d(h))
            .zip(c2.steps_iter())
        {
            assert_eq!(s1, s2)
        }
    }
}

#[test]
fn curve_on_demand_extrapolation() {
    let dmin: Vec<u64> = vec![1, 2, 12, 15, 18, 21];
    let mut curve = arrival::Curve::from_iter(dmin.iter().map(|t| Duration::from(*t)));

    let od_curve = arrival::ExtrapolatingCurve::new(curve.clone());

    let horizon = 1000;

    curve.extrapolate(d(horizon));

    for delta in 0..=horizon {
        assert_eq!(
            curve.number_arrivals(d(delta)),
            od_curve.number_arrivals(d(delta))
        )
    }

    for (s1, s2) in curve
        .steps_iter()
        .take_while(|s1| *s1 <= d(horizon))
        .zip(od_curve.steps_iter())
    {
        assert_eq!(s1, s2)
    }

    brute_force_iter_check(&curve);
    brute_force_iter_check(&od_curve);
}

#[test]
fn curve_on_demand_extrapolation_jitter_propagation() {
    let dmin: Vec<u64> = vec![1, 2, 12, 15, 18, 21];
    let mut curve = arrival::Curve::from_iter(dmin.iter().map(|t| Duration::from(*t)));
    let od_curve = arrival::ExtrapolatingCurve::new(curve.clone());

    let horizon = 200;
    curve.extrapolate(d(horizon));

    let jitters: Vec<u64> = vec![2, 5, 10, 13, 17, 19, 21, 123];
    for j in jitters.iter() {
        let c1 = curve.clone_with_jitter(d(*j));
        let c2 = od_curve.clone_with_jitter(d(*j));

        for delta in 0..=(horizon - j) {
            assert_eq!(c1.number_arrivals(d(delta)), c2.number_arrivals(d(delta)))
        }

        for (s1, s2) in c1
            .steps_iter()
            .take_while(|s1| *s1 <= d(horizon - j))
            .zip(c2.steps_iter())
        {
            assert_eq!(s1, s2)
        }

        brute_force_iter_check(&c1);
        brute_force_iter_check(&c2);
    }
}

#[test]
fn curve_on_demand_extrapolation_jitter_propagation_single() {
    let dmin: Vec<Duration> = vec![d(10)];
    let periodic = arrival::Periodic { period: dmin[0] };
    let mut curve = arrival::Curve::from_iter(dmin.iter().copied());
    let od_curve = arrival::ExtrapolatingCurve::new(curve.clone());

    let horizon = 200;
    curve.extrapolate(d(horizon));

    let jitters: Vec<u64> = vec![2, 5, 10, 13, 17, 19, 21, 123];
    for j in jitters.iter() {
        let c1 = curve.clone_with_jitter(d(*j));
        let c2 = od_curve.clone_with_jitter(d(*j));
        let c3 = periodic.clone_with_jitter(d(*j));

        for delta in 0..=(horizon - j) {
            assert_eq!(c1.number_arrivals(d(delta)), c3.number_arrivals(d(delta)));
            assert_eq!(c1.number_arrivals(d(delta)), c2.number_arrivals(d(delta)));
        }

        for (s1, s2) in c1
            .steps_iter()
            .take_while(|s1| *s1 <= d(horizon - j))
            .zip(c2.steps_iter())
        {
            assert_eq!(s1, s2)
        }
    }
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
