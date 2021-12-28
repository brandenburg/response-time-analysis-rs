pub mod arrival;
pub mod demand;
pub mod fixed_point;
pub mod supply;
pub mod time;
pub mod wcet;

pub mod ros2;

#[cfg(test)]
mod tests {
    use crate::arrival::{self, ArrivalBound};
    use crate::demand::{self, RequestBound};
    use crate::ros2;
    use crate::supply::{self, SupplyBound};
    use crate::time::{Duration, Offset, Service};
    use crate::wcet::{self, JobCostModel};
    use assert_approx_eq::assert_approx_eq;

    use std::iter::FromIterator;

    // helper function for typed duration values
    fn d(val: u64) -> Duration {
        Duration::from(val)
    }

    // helper function for vectors of typed duration values
    fn dv(vals: &[u64]) -> Vec<Duration> {
        vals.iter().map(|t| d(*t)).collect()
    }

    // helper function for typed service values
    fn s(val: u64) -> Service {
        Service::from(val)
    }

    // helper function for vectors of typed service values
    fn sv(vals: &[u64]) -> Vec<Service> {
        vals.iter().map(|t| s(*t)).collect()
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

    fn brute_force_iter_check<T: ArrivalBound>(ab: &T) {
        let si100 = ab.steps_iter().take(100);
        let bf100 = ab.brute_force_steps_iter().take(100);

        for (s1, s2) in si100.zip(bf100) {
            assert_eq!(s1, s2)
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
    fn periodic_supply() {
        let r = supply::Periodic {
            period: d(5),
            budget: s(3),
        };

        assert_eq!(r.provided_service(d(0)), s(0));
        assert_eq!(r.provided_service(d(1)), s(0));
        assert_eq!(r.provided_service(d(2)), s(0));
        assert_eq!(r.provided_service(d(3)), s(0));
        assert_eq!(r.provided_service(d(4)), s(0));
        assert_eq!(r.provided_service(d(5)), s(1));
        assert_eq!(r.provided_service(d(6)), s(2));
        assert_eq!(r.provided_service(d(7)), s(3));
        assert_eq!(r.provided_service(d(8)), s(3));
        assert_eq!(r.provided_service(d(9)), s(3));
        assert_eq!(r.provided_service(d(10)), s(4));
        assert_eq!(r.provided_service(d(11)), s(5));
        assert_eq!(r.provided_service(d(12)), s(6));
        assert_eq!(r.provided_service(d(13)), s(6));
        assert_eq!(r.provided_service(d(14)), s(6));
        assert_eq!(r.provided_service(d(15)), s(7));

        assert_eq!(r.service_time(s(0)), d(0));
        assert_eq!(r.service_time(s(1)), d(5));
        assert_eq!(r.service_time(s(2)), d(6));
        assert_eq!(r.service_time(s(3)), d(7));
        assert_eq!(r.service_time(s(4)), d(10));
        assert_eq!(r.service_time(s(5)), d(11));
        assert_eq!(r.service_time(s(6)), d(12));
        assert_eq!(r.service_time(s(7)), d(15));
        assert_eq!(r.service_time(s(8)), d(16));
        assert_eq!(r.service_time(s(9)), d(17));
        assert_eq!(r.service_time(s(10)), d(20));
        assert_eq!(r.service_time(s(11)), d(21));
        assert_eq!(r.service_time(s(12)), d(22));
        assert_eq!(r.service_time(s(13)), d(25));
        assert_eq!(r.service_time(s(14)), d(26));
        assert_eq!(r.service_time(s(15)), d(27));

        for cost in 1..1000 {
            let service_time = r.service_time(s(cost));
            let blackout_interference =
                service_time - Duration::from(r.provided_service(service_time));
            assert_eq!(blackout_interference + d(cost), service_time);
        }

        for x in 1..1000 {
            let st = r.service_time(s(x));
            assert_eq!(r.provided_service(st), s(x));
            assert!(r.provided_service(st - d(1)) < s(x));
        }
    }

    #[test]
    fn periodic_supply2() {
        for period in 2..112 {
            for budget in 1..=period {
                let cr = supply::Periodic::new(s(budget), d(period));
                for x in 1..1000 {
                    let st = cr.service_time(s(x));
                    assert_eq!(cr.provided_service(st), s(x));
                    assert!(cr.provided_service(st - d(1)) < s(x));
                }
            }
        }
    }

    #[test]
    fn constrained_supply_equiv() {
        let cr = supply::Constrained {
            period: d(5),
            budget: s(3),
            deadline: d(5),
        };
        let r = supply::Periodic {
            period: d(5),
            budget: s(3),
        };

        for delta in 1..1000 {
            assert_eq!(cr.provided_service(d(delta)), r.provided_service(d(delta)));
        }

        for cost in 1..1000 {
            assert_eq!(cr.service_time(s(cost)), r.service_time(s(cost)));
        }
    }

    #[test]
    fn constrained_supply() {
        let cr = supply::Constrained {
            period: d(11),
            budget: s(2),
            deadline: d(5),
        };

        assert_eq!(cr.provided_service(d(0)), s(0));
        assert_eq!(cr.provided_service(d(1)), s(0));
        assert_eq!(cr.provided_service(d(2)), s(0));
        assert_eq!(cr.provided_service(d(3)), s(0));
        assert_eq!(cr.provided_service(d(4)), s(0));
        assert_eq!(cr.provided_service(d(5)), s(0));
        assert_eq!(cr.provided_service(d(6)), s(0));
        assert_eq!(cr.provided_service(d(7)), s(0));
        assert_eq!(cr.provided_service(d(8)), s(0));
        assert_eq!(cr.provided_service(d(9)), s(0));
        assert_eq!(cr.provided_service(d(10)), s(0));
        assert_eq!(cr.provided_service(d(11)), s(0));
        assert_eq!(cr.provided_service(d(12)), s(0));
        assert_eq!(cr.provided_service(d(13)), s(1));
        assert_eq!(cr.provided_service(d(14)), s(2));
        assert_eq!(cr.provided_service(d(15)), s(2));
        assert_eq!(cr.provided_service(d(17)), s(2));
        assert_eq!(cr.provided_service(d(18)), s(2));
        assert_eq!(cr.provided_service(d(19)), s(2));
        assert_eq!(cr.provided_service(d(20)), s(2));
        assert_eq!(cr.provided_service(d(21)), s(2));
        assert_eq!(cr.provided_service(d(22)), s(2));
        assert_eq!(cr.provided_service(d(23)), s(2));
        assert_eq!(cr.provided_service(d(24)), s(3));
        assert_eq!(cr.provided_service(d(25)), s(4));
        assert_eq!(cr.provided_service(d(26)), s(4));

        for cost in 1..1000 {
            let service_time = cr.service_time(s(cost));
            let blackout_interference =
                service_time - Duration::from(cr.provided_service(service_time));
            assert_eq!(blackout_interference + d(cost), service_time);
        }
    }

    #[test]
    fn constrained_supply2() {
        let cr = supply::Constrained {
            period: d(100),
            budget: s(7),
            deadline: d(10),
        };

        assert_eq!(cr.provided_service(d(93)), s(0));
        assert_eq!(cr.provided_service(d(94)), s(0));
        assert_eq!(cr.provided_service(d(95)), s(0));
        assert_eq!(cr.provided_service(d(96)), s(0));
        assert_eq!(cr.provided_service(d(97)), s(1));
        assert_eq!(cr.provided_service(d(98)), s(2));
        assert_eq!(cr.provided_service(d(99)), s(3));
        assert_eq!(cr.provided_service(d(100)), s(4));
        assert_eq!(cr.provided_service(d(101)), s(5));
        assert_eq!(cr.provided_service(d(102)), s(6));
        assert_eq!(cr.provided_service(d(103)), s(7));
        assert_eq!(cr.provided_service(d(104)), s(7));

        for x in 1..1000 {
            let st = cr.service_time(s(x));
            assert_eq!(cr.provided_service(st), s(x));
            assert!(cr.provided_service(st - d(1)) < s(x));
        }

        for cost in 1..1000 {
            let service_time = cr.service_time(s(cost));
            let blackout_interference =
                service_time - Duration::from(cr.provided_service(service_time));
            assert_eq!(blackout_interference + d(cost), service_time);
        }
    }

    #[test]
    fn constrained_supply3() {
        for period in 2..29 {
            for deadline in 1..=period {
                for budget in 1..=deadline {
                    let cr = supply::Constrained::new(s(budget), d(deadline), d(period));
                    for x in 1..1000 {
                        let st = cr.service_time(s(x));
                        assert_eq!(cr.provided_service(st), s(x));
                        assert!(cr.provided_service(st - d(1)) < s(x));
                    }
                }
            }
        }
    }

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
        let cf2 =
            wcet::ExtrapolatingCurve::new(wcet::Curve::from_iter(wcets2.iter().map(|t| s(*t))));
        assert_eq!(cf2.cost_of_jobs(11), s(174));
        assert_eq!(cf2.cost_of_jobs(12), s(319));
        assert_eq!(cf2.cost_of_jobs(4), s(153));
        assert_eq!(cf2.cost_of_jobs(9), s(168));
        assert_eq!(cf2.cost_of_jobs(4 + 9), s(153 + 168));
    }

    #[test]
    fn ros2_event_source() {
        let sbf = supply::Periodic {
            period: d(5),
            budget: s(3),
        };

        let rbf = demand::RBF {
            wcet: wcet::Scalar::from(s(2)),
            arrival_bound: arrival::Sporadic {
                jitter: d(2),
                min_inter_arrival: d(5),
            },
        };

        let result = ros2::rta_event_source(&sbf, &rbf, d(100));

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), d(7));
    }

    #[test]
    fn ros2_never() {
        let sbf = supply::Periodic {
            period: d(5),
            budget: s(3),
        };

        let rbf = demand::RBF {
            wcet: wcet::Scalar::from(s(2)),
            arrival_bound: arrival::Never {},
        };

        let result = ros2::rta_event_source(&sbf, &rbf, d(100));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), d(0));
    }

    #[test]
    fn ros2_timer_periodic() {
        let sbf = supply::Periodic {
            period: d(5),
            budget: s(3),
        };

        // use RBF with boxed parameters, just because we can
        let rbf = demand::RBF {
            wcet: Box::new(wcet::Scalar::from(s(1))),
            arrival_bound: Box::new(arrival::Periodic { period: d(10) }),
        };

        let interference = demand::Aggregate::new(vec![
            demand::RBF {
                wcet: wcet::Scalar::from(s(1)),
                arrival_bound: arrival::Periodic { period: d(10) },
            },
            demand::RBF {
                wcet: wcet::Scalar::from(s(3)),
                arrival_bound: arrival::Periodic { period: d(20) },
            },
        ]);

        let result = ros2::rta_timer(&sbf, &rbf, &interference, s(0), d(100));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), d(12));

        let result2 = ros2::rta_timer(&sbf, &rbf, &interference, s(4), d(100));
        assert!(result2.is_ok());
        assert_eq!(result2.unwrap(), d(20));
    }

    fn brute_force_rbf_steps<'a, T: RequestBound>(
        rbf: &'a T,
    ) -> Box<dyn Iterator<Item = Duration> + 'a> {
        Box::new((1..).map(Duration::from).filter(move |delta| {
            let n = rbf.service_needed(*delta);
            let m = rbf.service_needed(*delta - d(1));
            assert!(m <= n, "demand must be monotonic");
            m < n
        }))
    }

    fn check_rbf_steps<T: RequestBound>(rbf: &T, horizon: Duration) {
        let bf_steps = brute_force_rbf_steps(rbf);
        let steps = rbf.steps_iter();
        for (bf_step, step) in bf_steps.zip(steps) {
            assert_eq!(bf_step, step);
            if bf_step > horizon {
                break;
            }
        }
    }

    #[test]
    fn ros2_timer_sporadic() {
        let sbf = supply::Periodic {
            period: d(5),
            budget: s(3),
        };

        let rbf = demand::RBF {
            wcet: wcet::Scalar::from(s(1)),
            arrival_bound: arrival::Periodic { period: d(10) },
        };

        let interference: Vec<Box<dyn RequestBound>> = vec![
            Box::new(demand::RBF {
                wcet: wcet::Scalar::from(s(1)),
                arrival_bound: arrival::Periodic { period: d(10) },
            }),
            Box::new(demand::RBF {
                wcet: wcet::Scalar::from(s(3)),
                arrival_bound: arrival::Sporadic {
                    min_inter_arrival: d(20),
                    jitter: d(10),
                },
            }),
        ];

        check_rbf_steps(&interference[0], d(10005));
        check_rbf_steps(&interference[1], d(10005));

        let result = ros2::rta_timer(
            &sbf,
            &rbf,
            &demand::Slice::of(&interference[0..2]),
            s(0),
            d(100),
        );
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), d(17));
    }

    #[test]
    fn ros2_pp_callback() {
        let sbf = supply::Periodic {
            period: d(5),
            budget: s(3),
        };

        let rbf = demand::RBF {
            wcet: wcet::Scalar::from(s(1)),
            arrival_bound: arrival::Periodic { period: d(10) },
        };

        let interference = demand::Aggregate::new(vec![
            demand::RBF {
                wcet: wcet::Scalar::from(s(1)),
                arrival_bound: arrival::Periodic { period: d(10) },
            },
            demand::RBF {
                wcet: wcet::Scalar::from(s(3)),
                arrival_bound: arrival::Periodic { period: d(20) },
            },
        ]);

        check_rbf_steps(&rbf, d(10005));
        check_rbf_steps(&interference, d(10005));

        let result = ros2::rta_polling_point_callback(&sbf, &rbf, &interference, d(100));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), d(12));
    }

    #[test]
    fn ros2_chain() {
        // Box the supply just to check that it's possible
        let sbf: Box<dyn SupplyBound> = Box::new(supply::Periodic {
            period: d(5),
            budget: s(3),
        });

        let chain1_wcet = vec![1, 2, 3];
        let chain2_wcet = vec![1, 1, 1];
        let chain3_wcet = vec![2, 1];

        let total1: u64 = chain1_wcet.iter().sum();
        let total2: u64 = chain2_wcet.iter().sum();
        let total3: u64 = chain3_wcet.iter().sum();

        let rbfs = vec![
            Box::new(demand::RBF {
                wcet: wcet::Scalar::from(s(total1)),
                arrival_bound: arrival::Periodic { period: d(25) },
            }) as Box<dyn RequestBound>,
            Box::new(demand::RBF {
                wcet: wcet::Scalar::from(s(total2)),
                arrival_bound: arrival::Sporadic {
                    min_inter_arrival: d(20),
                    jitter: d(25),
                },
            }),
            Box::new(demand::RBF {
                wcet: wcet::Scalar::from(s(total3)),
                arrival_bound: arrival::Sporadic {
                    min_inter_arrival: d(20),
                    jitter: d(25),
                },
            }),
        ];

        check_rbf_steps(&rbfs[0], d(10005));
        check_rbf_steps(&rbfs[1], d(10005));
        check_rbf_steps(&rbfs[2], d(10005));

        let other_chains = demand::Slice::of(&rbfs[1..]);
        let chain_ua = &rbfs[0];
        let chain_prefix = demand::RBF {
            wcet: wcet::Scalar::from(s(1 + 2)),
            arrival_bound: arrival::Periodic { period: d(25) },
        };
        let chain_last_callback = demand::RBF {
            wcet: wcet::Scalar::from(s(3)),
            arrival_bound: arrival::Periodic { period: d(25) },
        };

        check_rbf_steps(&other_chains, d(10005));
        check_rbf_steps(&chain_prefix, d(10005));

        let result = ros2::rta_processing_chain(
            &sbf,
            &chain_last_callback,
            &chain_prefix,
            &chain_ua,
            &other_chains,
            d(1000),
        );
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), d(72));
    }

    #[test]
    fn ros2_chain2() {
        let sbf = supply::Periodic {
            period: d(5),
            budget: s(3),
        };

        let chain1_wcet = vec![1, 2, 3];
        let chain2_wcet = vec![1, 1, 1];
        let chain3_wcet = vec![2, 1];

        let prefix1: u64 = chain1_wcet[0..2].iter().sum();
        let total2: u64 = chain2_wcet.iter().sum();
        let total3: u64 = chain3_wcet.iter().sum();

        let chain1_prefix = demand::RBF {
            wcet: wcet::Scalar::from(s(prefix1)),
            arrival_bound: arrival::Periodic { period: d(25) },
        };

        let chain1_suffix = demand::RBF {
            wcet: wcet::Scalar::from(s(chain1_wcet[2])),
            arrival_bound: arrival::Periodic { period: d(25) },
        };

        let other_chains = demand::Aggregate::new(vec![
            Box::new(demand::RBF {
                wcet: wcet::Scalar::from(s(total2)),
                arrival_bound: arrival::Sporadic {
                    min_inter_arrival: d(20),
                    jitter: d(25),
                },
            }),
            Box::new(demand::RBF {
                wcet: wcet::Scalar::from(s(total3)),
                arrival_bound: arrival::Sporadic {
                    min_inter_arrival: d(20),
                    jitter: d(25),
                },
            }),
        ]);

        let all_other_callbacks = demand::Aggregate::new(vec![
            Box::new(other_chains) as Box<dyn RequestBound>,
            Box::new(chain1_prefix),
        ]);

        check_rbf_steps(&all_other_callbacks, d(10005));
        check_rbf_steps(&chain1_suffix, d(10005));

        let result =
            ros2::rta_polling_point_callback(&sbf, &chain1_suffix, &all_other_callbacks, d(1000));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), d(72));
    }

    #[test]
    fn ros2_rr_subchain_analysis() {
        // keep it simple to reason about
        let sbf = supply::Dedicated::new();

        let arrival_models: Vec<Box<dyn ArrivalBound>> = vec![
            Box::new(arrival::Periodic::new(d(100))),
            Box::new(arrival::Periodic::new(d(10000))),
            Box::new(arrival::Periodic::new(d(1000))),
        ];

        let cost_models: Vec<Box<dyn JobCostModel>> = vec![
            Box::new(wcet::Scalar::from(s(10))),
            Box::new(wcet::Scalar::from(s(230))),
            Box::new(wcet::Scalar::from(s(190))),
        ];

        let assumed_response_time_bounds = vec![
            // (before polling point) + (own polling point)
            d((10 * 10) + (190 + 230 + 10)),
            d((10) + (10 + 190 + 230)),
            d((10) + (10 + 230 + 190)),
        ];

        let unknown = ros2::rr::CallbackType::PolledUnknownPrio;

        let callbacks = vec![
            ros2::rr::Callback::new(
                assumed_response_time_bounds[0],
                &arrival_models[0],
                &cost_models[0],
                unknown,
            ),
            ros2::rr::Callback::new(
                assumed_response_time_bounds[1],
                &arrival_models[1],
                &cost_models[1],
                unknown,
            ),
            ros2::rr::Callback::new(
                assumed_response_time_bounds[2],
                &arrival_models[2],
                &cost_models[2],
                unknown,
            ),
        ];

        let workload = &callbacks[..];

        // single-callback "chains"
        let sc0 = [&callbacks[0]];
        let sc1 = [&callbacks[1]];
        let sc2 = [&callbacks[2]];

        let b0 = ros2::rr::rta_subchain(&sbf, workload, &sc0[..], d(100000))
            .expect("no fixed point found");
        let b1 = ros2::rr::rta_subchain(&sbf, workload, &sc1[..], d(100000))
            .expect("no fixed point found");
        let b2 = ros2::rr::rta_subchain(&sbf, workload, &sc2[..], d(100000))
            .expect("no fixed point found");

        assert_eq!(b0, assumed_response_time_bounds[0]);
        assert_eq!(b1, assumed_response_time_bounds[1]);
        assert_eq!(b2, assumed_response_time_bounds[2]);
    }

    #[test]
    fn ros2_bw_subchain_analysis() {
        // keep it simple to reason about
        let sbf = supply::Dedicated::new();

        let arrival_models: Vec<Box<dyn ArrivalBound>> = vec![
            Box::new(arrival::Periodic::new(d(100))),
            Box::new(arrival::Periodic::new(d(10000))),
            Box::new(arrival::Periodic::new(d(1000))),
        ];

        let cost_models: Vec<Box<dyn JobCostModel>> = vec![
            Box::new(wcet::Scalar::from(s(10))),
            Box::new(wcet::Scalar::from(s(230))),
            Box::new(wcet::Scalar::from(s(190))),
        ];

        let assumed_response_time_bounds = vec![
            d(190 + 230 + 10),
            d(3 * 10 + 190 + 230 - 1),
            d(3 * 10 + 190 + 230 - 1),
        ];

        let unknown = ros2::rr::CallbackType::PolledUnknownPrio;

        let callbacks = vec![
            ros2::bw::Callback::new(
                assumed_response_time_bounds[0],
                &arrival_models[0],
                &cost_models[0],
                unknown,
            ),
            ros2::bw::Callback::new(
                assumed_response_time_bounds[1],
                &arrival_models[1],
                &cost_models[1],
                unknown,
            ),
            ros2::bw::Callback::new(
                assumed_response_time_bounds[2],
                &arrival_models[2],
                &cost_models[2],
                unknown,
            ),
        ];

        let workload = &callbacks[..];

        // single-callback "chains"
        let sc0 = [&callbacks[0]];
        let sc1 = [&callbacks[1]];
        let sc2 = [&callbacks[2]];

        let b0 = ros2::bw::rta_subchain(&sbf, workload, &sc0[..], d(100000))
            .expect("no fixed point found");
        let b1 = ros2::bw::rta_subchain(&sbf, workload, &sc1[..], d(100000))
            .expect("no fixed point found");
        let b2 = ros2::bw::rta_subchain(&sbf, workload, &sc2[..], d(100000))
            .expect("no fixed point found");

        assert_eq!(b0, assumed_response_time_bounds[0]);
        assert_eq!(b1, assumed_response_time_bounds[1]);
        assert_eq!(b2, assumed_response_time_bounds[2]);
    }

    #[test]
    fn curve_extrapolation() {
        let dmin: Vec<u64> = vec![1, 2, 12, 15, 18, 21];
        let mut curve = arrival::Curve::from_iter(dmin.iter().map(|t| d(*t)));

        curve.extrapolate(d(500));

        let dmin_ref: Vec<u64> = vec![
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
            0, 1, 2, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 4, 4, 4, 5, 5, 5, 6, 6, 6, 7, 7, 7, 7, 7, 7, 8,
            8, 8, 9, 9, 9, 10, 10, 10, 10, 10, 10, 11, 11, 11, 12, 12, 12, 13, 13, 13, 13, 13, 13,
            14, 14, 14, 15, 15, 15, 16, 16, 16, 16, 16, 16, 17, 17, 17, 18, 18, 18, 19, 19, 19, 19,
            19, 19, 20, 20, 20, 21, 21, 21, 22, 22, 22, 22, 22, 22, 23, 23, 23, 24, 24, 24, 25, 25,
            25, 25, 25, 25,
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
    fn singleton_subchain_rr() {
        let sbf = supply::Periodic {
            period: d(5),
            budget: s(3),
        };
        let arrivals = arrival::Periodic::new(d(100));
        let cost = wcet::Scalar::from(s(1));

        let callbacks = vec![ros2::rr::Callback::new(
            d(5),
            &arrivals,
            &cost,
            ros2::rr::CallbackType::Timer,
        )];

        let workload = &callbacks[..];

        // single-callback "subchain"
        let sc = [&callbacks[0]];

        let res = ros2::rr::rta_subchain(&sbf, workload, &sc[..], d(100000))
            .expect("no fixed point found");
        assert_eq!(res, d(5));
    }

    #[test]
    fn singleton_subchain_bw() {
        let sbf = supply::Periodic {
            period: d(5),
            budget: s(3),
        };
        let arrivals = arrival::Periodic::new(d(100));
        let cost = wcet::Scalar::from(s(1));

        let callbacks = vec![ros2::bw::Callback::new(
            d(5),
            &arrivals,
            &cost,
            ros2::rr::CallbackType::Timer,
        )];

        let workload = &callbacks[..];

        // single-callback "subchain"
        let sc = [&callbacks[0]];

        let res = ros2::bw::rta_subchain(&sbf, workload, &sc[..], d(100000))
            .expect("no fixed point found");
        assert_eq!(res, d(5));
    }
}
