pub mod arrivals;
pub mod demand;
pub mod fixed_point;
pub mod supply;
pub mod time;

pub mod ros2;

#[cfg(test)]
mod tests {
    use crate::arrivals::{self, ArrivalBound};
    use crate::demand::{self, JobCostModel, RequestBound};
    use crate::ros2;
    use crate::supply::{self, SupplyBound};
    use crate::time::Duration;
    use assert_approx_eq::assert_approx_eq;

    use std::iter::FromIterator;

    #[test]
    fn periodic_arrivals() {
        let a = arrivals::Periodic { period: 10 };
        assert_eq!(a.number_arrivals(0), 0);
        assert_eq!(a.number_arrivals(1), 1);
        assert_eq!(a.number_arrivals(8), 1);
        assert_eq!(a.number_arrivals(10), 1);
        assert_eq!(a.number_arrivals(11), 2);
        assert_eq!(a.number_arrivals(12), 2);
        assert_eq!(a.number_arrivals(13), 2);
        assert_eq!(a.number_arrivals(100), 10);
        assert_eq!(a.number_arrivals(105), 11);
    }

    #[test]
    fn period_arrivals_dmin() {
        let ab = arrivals::Periodic { period: 10 };
        let dmin_ref = vec![(0, 0), (1, 0), (2, 10), (3, 20), (4, 30), (5, 40)];
        let dmin = arrivals::delta_min_iter(&ab);

        for (should, is) in dmin_ref.iter().zip(dmin) {
            assert_eq!(*should, is);
        }
    }

    #[test]
    fn periodic_arrivals_via_unroll_sporadic() {
        let p = arrivals::Periodic { period: 10 };
        let s = arrivals::Sporadic::from(p);
        let a = arrivals::CurvePrefix::unroll_sporadic(&s, 1000);
        assert_eq!(a.number_arrivals(0), 0);
        assert_eq!(a.number_arrivals(1), 1);
        assert_eq!(a.number_arrivals(8), 1);
        assert_eq!(a.number_arrivals(10), 1);
        assert_eq!(a.number_arrivals(11), 2);
        assert_eq!(a.number_arrivals(12), 2);
        assert_eq!(a.number_arrivals(13), 2);
        assert_eq!(a.number_arrivals(100), 10);
        assert_eq!(a.number_arrivals(101), 11);
        assert_eq!(a.number_arrivals(105), 11);
    }

    #[test]
    fn periodic_arrivals_via_sporadic() {
        let p = arrivals::Periodic { period: 10 };
        let s = arrivals::Sporadic::from(p);
        let a = arrivals::CurvePrefix::from(s);
        assert_eq!(a.number_arrivals(0), 0);
        assert_eq!(a.number_arrivals(1), 1);
        assert_eq!(a.number_arrivals(8), 1);
        assert_eq!(a.number_arrivals(10), 1);
        assert_eq!(a.number_arrivals(11), 2);
        assert_eq!(a.number_arrivals(12), 2);
        assert_eq!(a.number_arrivals(13), 2);
        assert_eq!(a.number_arrivals(100), 10);
        assert_eq!(a.number_arrivals(101), 11);
        assert_eq!(a.number_arrivals(105), 11);
    }
    #[test]
    fn periodic_arrivals_unrolled() {
        let p = arrivals::Periodic { period: 10 };
        let a = arrivals::CurvePrefix::from(p);
        assert_eq!(a.number_arrivals(0), 0);
        assert_eq!(a.number_arrivals(1), 1);
        assert_eq!(a.number_arrivals(8), 1);
        assert_eq!(a.number_arrivals(10), 1);
        assert_eq!(a.number_arrivals(11), 2);
        assert_eq!(a.number_arrivals(12), 2);
        assert_eq!(a.number_arrivals(13), 2);
        assert_eq!(a.number_arrivals(100), 10);
        assert_eq!(a.number_arrivals(101), 11);
        assert_eq!(a.number_arrivals(105), 11);
    }

    #[test]
    fn periodic_arrivals_from_trace() {
        let trace: Vec<u64> = vec![0, 10, 20, 30, 40];
        let a = arrivals::CurvePrefix::from_trace(trace.iter(), 10);
        assert_eq!(a.number_arrivals(0), 0);
        assert_eq!(a.number_arrivals(1), 1);
        assert_eq!(a.number_arrivals(8), 1);
        assert_eq!(a.number_arrivals(10), 1);
        assert_eq!(a.number_arrivals(11), 2);
        assert_eq!(a.number_arrivals(12), 2);
        assert_eq!(a.number_arrivals(13), 2);
        assert_eq!(a.number_arrivals(100), 10);
        assert_eq!(a.number_arrivals(101), 11);
        assert_eq!(a.number_arrivals(105), 11);
    }

    #[test]
    fn compare_periodic_arrivals() {
        let p = arrivals::Periodic { period: 10 };
        let s = arrivals::Sporadic::from(p);
        let a = arrivals::CurvePrefix::from(p);
        let b = arrivals::CurvePrefix::from(s);
        let trace: Vec<u64> = vec![0, 10, 20, 30, 40];
        let t = arrivals::CurvePrefix::from_trace(trace.iter(), 2);
        for delta in 0..1000 {
            assert_eq!(a.number_arrivals(delta), p.number_arrivals(delta));
            assert_eq!(s.number_arrivals(delta), p.number_arrivals(delta));
            assert_eq!(a.number_arrivals(delta), b.number_arrivals(delta));
            assert_eq!(p.number_arrivals(delta), t.number_arrivals(delta));
        }
    }

    fn brute_force_iter_check<T: ArrivalBound>(ab: &T) {
        let si100 = ab.steps_iter().take(100);
        let bf100 = ab.brute_force_steps_iter().take(100);

        for (s1, s2) in si100.zip(bf100) {
            assert_eq!(s1, s2)
        }
    }

    #[test]
    fn periodic_iter() {
        let p = arrivals::Periodic { period: 10 };
        let steps: Vec<_> = p.steps_iter().take(5).collect();
        assert_eq!(steps, [1, 11, 21, 31, 41]);
        brute_force_iter_check(&p);

        let p2 = arrivals::CurvePrefix::from(arrivals::Sporadic::from(p));
        let steps2: Vec<_> = p2.steps_iter().take(5).collect();
        assert_eq!(steps2, [1, 11, 21, 31, 41]);
    }

    #[test]
    fn sporadic_arrivals() {
        let a = arrivals::Sporadic {
            min_inter_arrival: 10,
            jitter: 3,
        };
        assert_eq!(a.number_arrivals(0), 0);
        assert_eq!(a.number_arrivals(1), 1);
        assert_eq!(a.number_arrivals(8), 2);
        assert_eq!(a.number_arrivals(10), 2);
        assert_eq!(a.number_arrivals(11), 2);
        assert_eq!(a.number_arrivals(100), 11);
        assert_eq!(a.number_arrivals(107), 11);
        assert_eq!(a.number_arrivals(108), 12);
        assert_eq!(a.number_arrivals(1108), 112);
    }

    #[test]
    fn sporadic_arrivals_from_trace() {
        let trace: Vec<u64> = vec![0, 7, 17, 27, 37, 47, 57, 67, 77, 87, 110, 117];
        let a = arrivals::CurvePrefix::from_trace(trace.iter(), 5);
        assert_eq!(a.number_arrivals(0), 0);
        assert_eq!(a.number_arrivals(1), 1);
        assert_eq!(a.number_arrivals(8), 2);
        assert_eq!(a.number_arrivals(10), 2);
        assert_eq!(a.number_arrivals(11), 2);
    }

    #[test]
    fn sporadic_arrivals_unrolled() {
        let s = arrivals::Sporadic {
            min_inter_arrival: 10,
            jitter: 3,
        };
        let a = arrivals::CurvePrefix::from(s);
        assert_eq!(a.number_arrivals(0), 0);
        assert_eq!(a.number_arrivals(1), 1);
        assert_eq!(a.number_arrivals(8), 2);
        assert_eq!(a.number_arrivals(10), 2);
        assert_eq!(a.number_arrivals(11), 2);
        assert_eq!(a.number_arrivals(100), 11);
        assert_eq!(a.number_arrivals(107), 11);
        assert_eq!(a.number_arrivals(108), 12);
        assert_eq!(a.number_arrivals(1108), 112);
    }

    #[test]
    fn sporadic_arrivals_large_jitter() {
        let a = arrivals::Sporadic {
            min_inter_arrival: 10,
            jitter: 16,
        };
        assert_eq!(a.number_arrivals(0), 0);
        assert_eq!(a.number_arrivals(1), 2);
        assert_eq!(a.number_arrivals(4), 2);
        assert_eq!(a.number_arrivals(5), 3);
    }

    #[test]
    fn compare_sporadic_arrivals() {
        let s = arrivals::Sporadic {
            min_inter_arrival: 10,
            jitter: 16,
        };
        let a = arrivals::CurvePrefix::from(s);
        for delta in 0..1000 {
            assert_eq!(a.number_arrivals(delta), s.number_arrivals(delta));
        }
    }

    #[test]
    fn sporadic_iter() {
        let s1 = arrivals::Sporadic {
            min_inter_arrival: 10,
            jitter: 3,
        };
        let steps1: Vec<_> = s1.steps_iter().take(6).collect();
        assert_eq!(steps1, [1, 8, 18, 28, 38, 48]);

        let s2 = arrivals::Sporadic {
            min_inter_arrival: 10,
            jitter: 16,
        };
        let steps2: Vec<_> = s2.steps_iter().take(6).collect();
        assert_eq!(steps2, [1, 5, 15, 25, 35, 45]);

        let s3 = arrivals::CurvePrefix::from(s1);
        let steps3: Vec<_> = s3.steps_iter().take(6).collect();
        assert_eq!(steps3, [1, 8, 18, 28, 38, 48]);

        let s4 = arrivals::CurvePrefix::from(s2);
        let steps4: Vec<_> = s4.steps_iter().take(6).collect();
        assert_eq!(steps4, [1, 5, 15, 25, 35, 45]);

        brute_force_iter_check(&s1);
        brute_force_iter_check(&s2);
        brute_force_iter_check(&s3);
        brute_force_iter_check(&s4);
    }

    #[test]
    fn poisson() {
        let p = arrivals::Poisson { rate: 0.01 };
        assert_approx_eq!(p.arrival_probability(100, 0), 0.368, 0.001);
        assert_approx_eq!(p.arrival_probability(100, 1), 0.368, 0.001);
        assert_approx_eq!(p.arrival_probability(100, 2), 0.184, 0.001);
        assert_approx_eq!(p.arrival_probability(100, 3), 0.061, 0.001);
        assert_approx_eq!(p.arrival_probability(100, 4), 0.015, 0.001);
        assert_approx_eq!(p.arrival_probability(100, 5), 0.003, 0.001);
        assert_approx_eq!(p.arrival_probability(100, 6), 0.0005, 0.001);
    }

    #[test]
    fn never_arrives() {
        let p = arrivals::Never {};
        assert_eq!(p.number_arrivals(10), 0);
        let steps: Vec<_> = p.steps_iter().collect();
        assert_eq!(steps, []);
        let prop = arrivals::Propagated {
            input_event_model: p,
            response_time_jitter: 3,
        };
        assert_eq!(prop.number_arrivals(10), 0);
        let steps: Vec<_> = p.steps_iter().collect();
        assert_eq!(steps, []);
        let dmin: Vec<_> = arrivals::nonzero_delta_min_iter(&p).collect();
        assert_eq!(dmin, []);
    }

    #[test]
    fn propagated_jitter() {
        let p = arrivals::Periodic { period: 10 };
        let s = arrivals::Sporadic {
            min_inter_arrival: 10,
            jitter: 3,
        };
        let prop = arrivals::Propagated {
            input_event_model: p,
            response_time_jitter: 3,
        };
        for t in 0..1000 {
            assert_eq!(s.number_arrivals(t), prop.number_arrivals(t));
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
            Box::new(arrivals::Sporadic {
                min_inter_arrival: 3,
                jitter: 0,
            }),
            Box::new(arrivals::Periodic { period: 5 }),
        ];

        let ab = &agg;

        assert_eq!(ab.number_arrivals(0), 0);
        assert_eq!(ab.number_arrivals(1), 2);
        assert_eq!(ab.number_arrivals(2), 2);
        assert_eq!(ab.number_arrivals(3), 2);
        assert_eq!(ab.number_arrivals(4), 3);
        assert_eq!(ab.number_arrivals(5), 3);
        assert_eq!(ab.number_arrivals(6), 4);
        assert_eq!(ab.number_arrivals(7), 5);
        assert_eq!(ab.number_arrivals(8), 5);
        assert_eq!(ab.number_arrivals(9), 5);
        assert_eq!(ab.number_arrivals(10), 6);
        assert_eq!(ab.number_arrivals(11), 7);
        assert_eq!(ab.number_arrivals(12), 7);
        assert_eq!(ab.number_arrivals(13), 8);
        assert_eq!(ab.number_arrivals(14), 8);
        assert_eq!(ab.number_arrivals(15), 8);
        assert_eq!(ab.number_arrivals(16), 10);

        let steps: Vec<Duration> = ab.steps_iter().take_while(|x| *x < 17).collect();
        assert_eq!(steps, vec![1, 4, 6, 7, 10, 11, 13, 16]);
        brute_force_iter_check(ab);

        let a = &ab[0..1];
        assert_eq!(a.number_arrivals(10), 4);
        brute_force_iter_check(&a);

        let boxed = Box::new(agg);
        assert_eq!(boxed.number_arrivals(16), 10);
        brute_force_iter_check(&boxed);

        let a_boxed = &boxed[0..1];
        assert_eq!(a_boxed.number_arrivals(10), 4);

        let boxed_ptr: &dyn ArrivalBound = &boxed;
        assert_eq!(boxed_ptr.number_arrivals(10), 6);
    }

    #[test]
    fn periodic_supply() {
        let r = supply::Periodic {
            period: 5,
            budget: 3,
        };

        assert_eq!(r.provided_service(0), 0);
        assert_eq!(r.provided_service(1), 0);
        assert_eq!(r.provided_service(2), 0);
        assert_eq!(r.provided_service(3), 0);
        assert_eq!(r.provided_service(4), 0);
        assert_eq!(r.provided_service(5), 1);
        assert_eq!(r.provided_service(6), 2);
        assert_eq!(r.provided_service(7), 3);
        assert_eq!(r.provided_service(8), 3);
        assert_eq!(r.provided_service(9), 3);
        assert_eq!(r.provided_service(10), 4);
        assert_eq!(r.provided_service(11), 5);
        assert_eq!(r.provided_service(12), 6);
        assert_eq!(r.provided_service(13), 6);
        assert_eq!(r.provided_service(14), 6);
        assert_eq!(r.provided_service(15), 7);

        assert_eq!(r.service_time(0), 0);
        assert_eq!(r.service_time(1), 5);
        assert_eq!(r.service_time(2), 6);
        assert_eq!(r.service_time(3), 7);
        assert_eq!(r.service_time(4), 10);
        assert_eq!(r.service_time(5), 11);
        assert_eq!(r.service_time(6), 12);
        assert_eq!(r.service_time(7), 15);
        assert_eq!(r.service_time(8), 16);
        assert_eq!(r.service_time(9), 17);
        assert_eq!(r.service_time(10), 20);
        assert_eq!(r.service_time(11), 21);
        assert_eq!(r.service_time(12), 22);
        assert_eq!(r.service_time(13), 25);
        assert_eq!(r.service_time(14), 26);
        assert_eq!(r.service_time(15), 27);

        for cost in 1..1000 {
            let service_time = r.service_time(cost);
            let blackout_interference = service_time - r.provided_service(service_time);
            assert_eq!(blackout_interference + cost, service_time);
        }

        for x in 1..1000 {
            let st = r.service_time(x);
            assert_eq!(r.provided_service(st), x);
            assert!(r.provided_service(st - 1) < x);
        }
    }

    #[test]
    fn periodic_supply2() {
        for period in 2..112 {
            for budget in 1..=period {
                let cr = supply::Periodic { period, budget };
                for x in 1..1000 {
                    let st = cr.service_time(x);
                    assert_eq!(cr.provided_service(st), x);
                    assert!(cr.provided_service(st - 1) < x);
                }
            }
        }
    }

    #[test]
    fn constrained_supply_equiv() {
        let cr = supply::Constrained {
            period: 5,
            budget: 3,
            deadline: 5,
        };
        let r = supply::Periodic {
            period: 5,
            budget: 3,
        };

        for delta in 1..1000 {
            assert_eq!(cr.provided_service(delta), r.provided_service(delta));
        }

        for cost in 1..1000 {
            assert_eq!(cr.service_time(cost), r.service_time(cost));
        }
    }

    #[test]
    fn constrained_supply() {
        let cr = supply::Constrained {
            period: 11,
            budget: 2,
            deadline: 5,
        };

        assert_eq!(cr.provided_service(0), 0);
        assert_eq!(cr.provided_service(1), 0);
        assert_eq!(cr.provided_service(2), 0);
        assert_eq!(cr.provided_service(3), 0);
        assert_eq!(cr.provided_service(4), 0);
        assert_eq!(cr.provided_service(5), 0);
        assert_eq!(cr.provided_service(6), 0);
        assert_eq!(cr.provided_service(7), 0);
        assert_eq!(cr.provided_service(8), 0);
        assert_eq!(cr.provided_service(9), 0);
        assert_eq!(cr.provided_service(10), 0);
        assert_eq!(cr.provided_service(11), 0);
        assert_eq!(cr.provided_service(12), 0);
        assert_eq!(cr.provided_service(13), 1);
        assert_eq!(cr.provided_service(14), 2);
        assert_eq!(cr.provided_service(15), 2);
        assert_eq!(cr.provided_service(17), 2);
        assert_eq!(cr.provided_service(18), 2);
        assert_eq!(cr.provided_service(19), 2);
        assert_eq!(cr.provided_service(20), 2);
        assert_eq!(cr.provided_service(21), 2);
        assert_eq!(cr.provided_service(22), 2);
        assert_eq!(cr.provided_service(23), 2);
        assert_eq!(cr.provided_service(24), 3);
        assert_eq!(cr.provided_service(25), 4);
        assert_eq!(cr.provided_service(26), 4);

        for cost in 1..1000 {
            let service_time = cr.service_time(cost);
            let blackout_interference = service_time - cr.provided_service(service_time);
            assert_eq!(blackout_interference + cost, service_time);
        }
    }

    #[test]
    fn constrained_supply2() {
        let cr = supply::Constrained {
            period: 100,
            budget: 7,
            deadline: 10,
        };

        assert_eq!(cr.provided_service(93), 0);
        assert_eq!(cr.provided_service(94), 0);
        assert_eq!(cr.provided_service(95), 0);
        assert_eq!(cr.provided_service(96), 0);
        assert_eq!(cr.provided_service(97), 1);
        assert_eq!(cr.provided_service(98), 2);
        assert_eq!(cr.provided_service(99), 3);
        assert_eq!(cr.provided_service(100), 4);
        assert_eq!(cr.provided_service(101), 5);
        assert_eq!(cr.provided_service(102), 6);
        assert_eq!(cr.provided_service(103), 7);
        assert_eq!(cr.provided_service(104), 7);

        for x in 1..1000 {
            let st = cr.service_time(x);
            assert_eq!(cr.provided_service(st), x);
            assert!(cr.provided_service(st - 1) < x);
        }

        for cost in 1..1000 {
            let service_time = cr.service_time(cost);
            let blackout_interference = service_time - cr.provided_service(service_time);
            assert_eq!(blackout_interference + cost, service_time);
        }
    }

    #[test]
    fn constrained_supply3() {
        for period in 2..29 {
            for deadline in 1..=period {
                for budget in 1..=deadline {
                    let cr = supply::Constrained {
                        period,
                        budget,
                        deadline,
                    };
                    for x in 1..1000 {
                        let st = cr.service_time(x);
                        assert_eq!(cr.provided_service(st), x);
                        assert!(cr.provided_service(st - 1) < x);
                    }
                }
            }
        }
    }

    #[test]
    fn cost_models() {
        let wcet: Duration = 10;

        assert_eq!(wcet.cost_of_jobs(0), 0);
        assert_eq!(wcet.cost_of_jobs(3), 30);
        assert_eq!(wcet.cost_of_jobs(10), 100);
        let jobs1: Vec<_> = wcet.job_cost_iter().take(5).collect();
        assert_eq!(jobs1, [10, 10, 10, 10, 10]);

        let multi_frame: Vec<Duration> = vec![3, 2, 1];
        assert_eq!(multi_frame.cost_of_jobs(0), 0);
        assert_eq!(multi_frame.cost_of_jobs(1), 3);
        assert_eq!(multi_frame.cost_of_jobs(2), 5);
        assert_eq!(multi_frame.cost_of_jobs(3), 6);
        assert_eq!(multi_frame.cost_of_jobs(4), 9);
        assert_eq!(multi_frame.cost_of_jobs(5), 11);
        assert_eq!(multi_frame.cost_of_jobs(6), 12);
        let jobs2: Vec<_> = multi_frame.job_cost_iter().take(5).collect();
        assert_eq!(jobs2, [3, 2, 1, 3, 2]);

        let trace = vec![1, 1, 3, 1, 2, 2, 1, 3, 1, 0, 0, 3, 2, 0, 1, 1];
        let cf = demand::CostFunction::from_trace(trace.iter(), 3);
        assert_eq!(cf.cost_of_jobs(0), 0);
        assert_eq!(cf.cost_of_jobs(1), 3);
        assert_eq!(cf.cost_of_jobs(2), 5);
        assert_eq!(cf.cost_of_jobs(3), 6);
        assert_eq!(cf.cost_of_jobs(4), 9);
        assert_eq!(cf.cost_of_jobs(5), 11);
        assert_eq!(cf.cost_of_jobs(6), 12);
        let jobs3: Vec<_> = cf.job_cost_iter().take(10).collect();
        assert_eq!(jobs3, [3, 2, 1, 3, 2, 1, 3, 2, 1, 3]);

        let wcets = vec![15, 25, 30];
        let cf2 = demand::CostFunction::from_iter(wcets);
        assert_eq!(cf2.cost_of_jobs(0), 0);
        assert_eq!(cf2.cost_of_jobs(1), 15);
        assert_eq!(cf2.cost_of_jobs(2), 25);
        assert_eq!(cf2.cost_of_jobs(3), 30);
        assert_eq!(cf2.cost_of_jobs(4), 45);
        assert_eq!(cf2.cost_of_jobs(5), 55);
        assert_eq!(cf2.cost_of_jobs(6), 60);
        let jobs4: Vec<_> = cf2.job_cost_iter().take(10).collect();
        assert_eq!(jobs4, [15, 10, 5, 15, 10, 5, 15, 10, 5, 15]);
    }

    #[test]
    fn cost_extrapolation() {
        let wcets = vec![100, 101, 102, 103, 104, 105, 205, 206];
        let mut cf = demand::CostFunction::from_iter(wcets);
        assert_eq!(cf.cost_of_jobs(9), 306);
        cf.extrapolate(10);
        assert_eq!(cf.cost_of_jobs(9), 207);

	let wcets2 = vec![145,149,151,153,157,160,163,166,168,171,174];
	let mut cf2 = demand::CostFunction::from_iter(wcets2);
	assert_eq!(cf2.cost_of_jobs(11), 174);
	assert_eq!(cf2.cost_of_jobs(12), 319);
	assert_eq!(cf2.cost_of_jobs(4), 153);
	assert_eq!(cf2.cost_of_jobs(9), 168);
	assert_eq!(cf2.cost_of_jobs(4+9), 153+168);
    }

    #[test]
    fn ros2_event_source() {
        let sbf = supply::Periodic {
            period: 5,
            budget: 3,
        };

        let rbf = demand::RBF {
            wcet: 2,
            arrival_bound: arrivals::Sporadic {
                jitter: 2,
                min_inter_arrival: 5,
            },
        };

        let result = ros2::rta_event_source(&sbf, &rbf, 100);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 7);
    }

    #[test]
    fn ros2_never() {
        let sbf = supply::Periodic {
            period: 5,
            budget: 3,
        };

        let rbf = demand::RBF {
            wcet: 2,
            arrival_bound: arrivals::Never {},
        };

        let result = ros2::rta_event_source(&sbf, &rbf, 100);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    #[test]
    fn ros2_timer_periodic() {
        let sbf = supply::Periodic {
            period: 5,
            budget: 3,
        };

        // use RBF with boxed parameters, just because we can
        let rbf = demand::RBF {
            wcet: Box::new(1),
            arrival_bound: Box::new(arrivals::Periodic { period: 10 }),
        };

        let interference = vec![
            demand::RBF {
                wcet: 1,
                arrival_bound: arrivals::Periodic { period: 10 },
            },
            demand::RBF {
                wcet: 3,
                arrival_bound: arrivals::Periodic { period: 20 },
            },
        ];

        let result = ros2::rta_timer(&sbf, &rbf, &interference, 0, 100);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 12);

        let result2 = ros2::rta_timer(&sbf, &rbf, &interference, 4, 100);
        assert!(result2.is_ok());
        assert_eq!(result2.unwrap(), 20);
    }

    #[test]
    fn ros2_timer_sporadic() {
        let sbf = supply::Periodic {
            period: 5,
            budget: 3,
        };

        let rbf = demand::RBF {
            wcet: 1,
            arrival_bound: arrivals::Periodic { period: 10 },
        };

        let interference: Vec<Box<dyn RequestBound>> = vec![
            Box::new(demand::RBF {
                wcet: 1,
                arrival_bound: arrivals::Periodic { period: 10 },
            }),
            Box::new(demand::RBF {
                wcet: 3,
                arrival_bound: arrivals::Sporadic {
                    min_inter_arrival: 20,
                    jitter: 10,
                },
            }),
        ];

        let result = ros2::rta_timer(&sbf, &rbf, &interference[0..2], 0, 100);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 17);
    }

    #[test]
    fn ros2_pp_callback() {
        let sbf = supply::Periodic {
            period: 5,
            budget: 3,
        };

        let rbf = demand::RBF {
            wcet: 1,
            arrival_bound: arrivals::Periodic { period: 10 },
        };

        let interference = vec![
            demand::RBF {
                wcet: 1,
                arrival_bound: arrivals::Periodic { period: 10 },
            },
            demand::RBF {
                wcet: 3,
                arrival_bound: arrivals::Periodic { period: 20 },
            },
        ];

        let result = ros2::rta_polling_point_callback(&sbf, &rbf, &interference, 100);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 12);
    }

    #[test]
    fn ros2_chain() {
        // Box the supply just to check that it's possible
        let sbf: Box<dyn SupplyBound> = Box::new(supply::Periodic {
            period: 5,
            budget: 3,
        });

        let chain1_wcet = vec![1, 2, 3];
        let chain2_wcet = vec![1, 1, 1];
        let chain3_wcet = vec![2, 1];

        let total1: Duration = chain1_wcet.iter().sum();
        let total2: Duration = chain2_wcet.iter().sum();
        let total3: Duration = chain3_wcet.iter().sum();

        let all_chains: Vec<Box<dyn RequestBound>> = vec![
            Box::new(demand::RBF {
                wcet: total1,
                arrival_bound: arrivals::Periodic { period: 25 },
            }),
            Box::new(demand::RBF {
                wcet: total2,
                arrival_bound: arrivals::Sporadic {
                    min_inter_arrival: 20,
                    jitter: 25,
                },
            }),
            Box::new(demand::RBF {
                wcet: total3,
                arrival_bound: arrivals::Sporadic {
                    min_inter_arrival: 20,
                    jitter: 25,
                },
            }),
        ];

        let last_cb = *chain1_wcet.iter().last().unwrap();

        let result = ros2::rta_processing_chain(&sbf, &all_chains, last_cb, 1000);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 152);
    }

    #[test]
    fn ros2_chain2() {
        let sbf = supply::Periodic {
            period: 5,
            budget: 3,
        };

        let chain1_wcet = vec![1, 2, 3];
        let chain2_wcet = vec![1, 1, 1];
        let chain3_wcet = vec![2, 1];

        let prefix1: Duration = chain1_wcet[0..2].iter().sum();
        let total2: Duration = chain2_wcet.iter().sum();
        let total3: Duration = chain3_wcet.iter().sum();

        let chain1_prefix = demand::RBF {
            wcet: prefix1,
            arrival_bound: arrivals::Periodic { period: 25 },
        };

        let chain1_suffix = demand::RBF {
            wcet: chain1_wcet[2],
            arrival_bound: arrivals::Periodic { period: 25 },
        };

        let other_chains: Vec<Box<dyn RequestBound>> = vec![
            Box::new(demand::RBF {
                wcet: total2,
                arrival_bound: arrivals::Sporadic {
                    min_inter_arrival: 20,
                    jitter: 25,
                },
            }),
            Box::new(demand::RBF {
                wcet: total3,
                arrival_bound: arrivals::Sporadic {
                    min_inter_arrival: 20,
                    jitter: 25,
                },
            }),
        ];

        let all_other_callbacks: Vec<Box<dyn RequestBound>> =
            vec![Box::new(other_chains), Box::new(chain1_prefix)];

        let result = ros2::rta_polling_point_callback(&sbf, &chain1_suffix, &all_other_callbacks, 1000);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 72);
    }

    #[test]
    fn ros2_chain3() {
        let sbf = supply::Periodic {
            period: 5,
            budget: 3,
        };

        let chain1_wcet: Vec<Duration> = vec![1, 2, 3];
        let chain2_wcet = vec![1, 1, 1];
        let chain3_wcet = vec![2, 1];

        let total2: Duration = chain2_wcet.iter().sum();
        let total3: Duration = chain3_wcet.iter().sum();

        let chain1_arrivals = arrivals::Periodic { period: 25 };

        let other_chains: Vec<Box<dyn RequestBound>> = vec![
            Box::new(demand::RBF {
                wcet: total2,
                arrival_bound: arrivals::Sporadic {
                    min_inter_arrival: 20,
                    jitter: 25,
                },
            }),
            Box::new(demand::RBF {
                wcet: total3,
                arrival_bound: arrivals::Sporadic {
                    min_inter_arrival: 20,
                    jitter: 25,
                },
            }),
        ];

        let result = ros2::rta_processing_chain_window_aware(
            &sbf,
            chain1_wcet.iter().copied(),
            &chain1_arrivals,
            &other_chains,
            1000,
        );
        assert!(result.is_ok());
        // dbg!(result);
        // assert_eq!(result.unwrap(), 72);
    }

    #[test]
    fn curve_extrapolation() {
        let dmin: Vec<Duration> = vec![1, 2, 12, 15, 18, 21];
        let mut curve = arrivals::CurvePrefix::from_iter(dmin.iter().copied());

        // assert_eq!(curve.number_arrivals(22), 9);
        // assert_eq!(curve.number_arrivals(23), 10);
        // assert_eq!(curve.number_arrivals(25), 11);

        curve.extrapolate(500);

        let dmin_ref: Vec<Duration> = vec![
            0, 0, 1, 2, 12, 15, 18, 21, 27, 30, 33, 39, 42, 45, 51, 54, 57, 63, 66, 69, 75, 78, 81,
            87, 90, 93, 99, 102, 105, 111, 114, 117, 123, 126, 129, 135, 138, 141, 147, 150, 153,
            159, 162, 165, 171, 174, 177, 183, 186, 189, 195, 198, 201, 207, 210, 213, 219, 222,
            225, 231, 234, 237, 243, 246, 249, 255, 258, 261, 267, 270, 273, 279, 282, 285, 291,
            294, 297, 303, 306, 309, 315, 318, 321, 327, 330, 333, 339, 342, 345, 351, 354, 357,
            363, 366, 369, 375, 378, 381, 387, 390,
        ];

        for (x, dist) in dmin_ref.iter().enumerate() {
            if x > dmin.len() + 1 {
                let all_combinations: Vec<_> = (2..x)
                    .map(|k| (k, curve.min_distance(x - k + 1) + curve.min_distance(k)))
                    .collect();
                let y = all_combinations.iter().map(|(_, y)| *y).max().unwrap_or(0);
                assert_eq!(y, curve.min_distance(x));
            }
            assert_eq!(*dist, curve.min_distance(x));
        }

        let ab_ref: Vec<usize> = vec![
            0, 1, 2, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 4, 4, 4, 5, 5, 5, 6, 6, 6, 7, 7, 7, 7, 7, 7, 8,
            8, 8, 9, 9, 9, 10, 10, 10, 10, 10, 10, 11, 11, 11, 12, 12, 12, 13, 13, 13, 13, 13, 13,
            14, 14, 14, 15, 15, 15, 16, 16, 16, 16, 16, 16, 17, 17, 17, 18, 18, 18, 19, 19, 19, 19,
            19, 19, 20, 20, 20, 21, 21, 21, 22, 22, 22, 22, 22, 22, 23, 23, 23, 24, 24, 24, 25, 25,
            25, 25, 25, 25,
        ];

        for (delta, njobs) in ab_ref.iter().enumerate() {
            assert_eq!(*njobs, curve.number_arrivals(delta as Duration));
        }

        for (should, is) in (0..)
            .zip(dmin_ref.iter().copied())
            .zip(arrivals::delta_min_iter(&curve))
        {
            assert_eq!(should, is);
        }
    }

    #[test]
    fn curve_jitter_steps() {
        let dmin: Vec<Duration> = vec![1, 2, 12, 15, 18, 21];
        let curve = arrivals::CurvePrefix::from_iter(dmin.iter().copied());
        let curve_with_jitter = curve.clone_with_jitter(2);
        brute_force_iter_check(&curve);
        brute_force_iter_check(&curve_with_jitter);
    }

    #[test]
    fn curve_on_demand_jitter() {
        let dmin: Vec<Duration> = vec![1, 2, 12, 15, 18, 21];
        let mut curve = arrivals::CurvePrefix::from_iter(dmin.iter().copied());
        let horizon = 1000;
        curve.extrapolate(horizon);
        let od_curve = arrivals::ExtrapolatingCurvePrefix::new(curve.clone());

        let jitters: Vec<Duration> = vec![2, 5, 10, 13, 17, 19, 21, 123];
        for j in jitters.iter() {
            let c1 = curve.clone_with_jitter(*j);
            let c2 = od_curve.clone_with_jitter(*j);

            for delta in 0..=(horizon - j) {
                assert_eq!(c1.number_arrivals(delta), c2.number_arrivals(delta))
            }

            for (s1, s2) in c1
                .steps_iter()
                .take_while(|s1| *s1 <= horizon - j)
                .zip(c2.steps_iter())
            {
                assert_eq!(s1, s2)
            }
        }

        let mut c1 = curve.clone_with_jitter(1);
        let mut c2 = od_curve.clone_with_jitter(1);
        let mut h = horizon - 1;

        for j in jitters.iter() {
            c1 = c1.clone_with_jitter(*j);
            c2 = c2.clone_with_jitter(*j);
            h -= *j;

            for delta in 0..=h {
                assert_eq!(c1.number_arrivals(delta), c2.number_arrivals(delta))
            }

            for (s1, s2) in c1
                .steps_iter()
                .take_while(|s1| *s1 <= h)
                .zip(c2.steps_iter())
            {
                assert_eq!(s1, s2)
            }
        }
    }

    #[test]
    fn curve_on_demand_extrapolation() {
        let dmin: Vec<Duration> = vec![1, 2, 12, 15, 18, 21];
        let mut curve = arrivals::CurvePrefix::from_iter(dmin.iter().copied());

        let od_curve = arrivals::ExtrapolatingCurvePrefix::new(curve.clone());

        let horizon = 1000;

        curve.extrapolate(horizon);

        for delta in 0..=horizon {
            assert_eq!(
                curve.number_arrivals(delta),
                od_curve.number_arrivals(delta)
            )
        }

        for (s1, s2) in curve
            .steps_iter()
            .take_while(|s1| *s1 <= horizon)
            .zip(od_curve.steps_iter())
        {
            assert_eq!(s1, s2)
        }

        brute_force_iter_check(&curve);
        brute_force_iter_check(&od_curve);
    }

    #[test]
    fn curve_on_demand_extrapolation_jitter_propagation() {
        let dmin: Vec<Duration> = vec![1, 2, 12, 15, 18, 21];
        let mut curve = arrivals::CurvePrefix::from_iter(dmin.iter().copied());
        let od_curve = arrivals::ExtrapolatingCurvePrefix::new(curve.clone());

        let horizon = 200;
        curve.extrapolate(horizon);

        let jitters: Vec<Duration> = vec![2, 5, 10, 13, 17, 19, 21, 123];
        for j in jitters.iter() {
            let c1 = curve.clone_with_jitter(*j);
            let c2 = od_curve.clone_with_jitter(*j);

            for delta in 0..=(horizon - j) {
                assert_eq!(c1.number_arrivals(delta), c2.number_arrivals(delta))
            }

            for (s1, s2) in c1
                .steps_iter()
                .take_while(|s1| *s1 <= horizon - j)
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
        let dmin: Vec<Duration> = vec![10];
        let periodic = arrivals::Periodic { period: dmin[0] };
        let mut curve = arrivals::CurvePrefix::from_iter(dmin.iter().copied());
        let od_curve = arrivals::ExtrapolatingCurvePrefix::new(curve.clone());

        let horizon = 200;
        curve.extrapolate(horizon);

        let jitters: Vec<Duration> = vec![2, 5, 10, 13, 17, 19, 21, 123];
        for j in jitters.iter() {
            let c1 = curve.clone_with_jitter(*j);
            let c2 = od_curve.clone_with_jitter(*j);
            let c3 = periodic.clone_with_jitter(*j);

            for delta in 0..=(horizon - j) {
                assert_eq!(c1.number_arrivals(delta), c3.number_arrivals(delta));
                assert_eq!(c1.number_arrivals(delta), c2.number_arrivals(delta));
            }

            for (s1, s2) in c1
                .steps_iter()
                .take_while(|s1| *s1 <= horizon - j)
                .zip(c2.steps_iter())
            {
                assert_eq!(s1, s2)
            }
        }
    }
}
