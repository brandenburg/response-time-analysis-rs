pub mod arrivals;
pub mod time;
pub mod supply;
pub mod analysis;
pub mod demand;

pub mod ros2;

#[cfg(test)]
mod tests {
    use crate::time::Duration;
    use crate::arrivals::{self, ArrivalBound};
    use crate::supply::{self, SupplyBound};
    use crate::demand::{self, RequestBound, JobCostModel};
    use crate::ros2;
    use assert_approx_eq::assert_approx_eq;
    
    use std::iter::FromIterator;

    #[test]
    fn periodic_arrivals() {
        let a = arrivals::Periodic{ period: 10 };
        assert_eq!(a.number_arrivals(0),  0);
        assert_eq!(a.number_arrivals(1),  1);
        assert_eq!(a.number_arrivals(8),  1);        
        assert_eq!(a.number_arrivals(10),  1);
        assert_eq!(a.number_arrivals(11),  2);
        assert_eq!(a.number_arrivals(12),  2);
        assert_eq!(a.number_arrivals(13),  2);
        assert_eq!(a.number_arrivals(100), 10);
        assert_eq!(a.number_arrivals(105), 11);
    }

    #[test]
    fn periodic_arrivals_via_unroll_sporadic() {
        let p = arrivals::Periodic{ period: 10 };
        let s = arrivals::Sporadic::from(p);
        let a = arrivals::CurvePrefix::unroll_sporadic(&s, 1000);
        assert_eq!(a.number_arrivals(0),  0);
        assert_eq!(a.number_arrivals(1),  1);
        assert_eq!(a.number_arrivals(8),  1);        
        assert_eq!(a.number_arrivals(10),  1);
        assert_eq!(a.number_arrivals(11),  2);
        assert_eq!(a.number_arrivals(12),  2);
        assert_eq!(a.number_arrivals(13),  2);
        assert_eq!(a.number_arrivals(100), 10);
        assert_eq!(a.number_arrivals(101), 11);
        assert_eq!(a.number_arrivals(105), 11);
    }

    #[test]
    fn periodic_arrivals_via_sporadic() {
        let p = arrivals::Periodic{ period: 10 };
        let s = arrivals::Sporadic::from(p);
        let a = arrivals::CurvePrefix::from(s);
        assert_eq!(a.number_arrivals(0),  0);
        assert_eq!(a.number_arrivals(1),  1);
        assert_eq!(a.number_arrivals(8),  1);        
        assert_eq!(a.number_arrivals(10),  1);
        assert_eq!(a.number_arrivals(11),  2);
        assert_eq!(a.number_arrivals(12),  2);
        assert_eq!(a.number_arrivals(13),  2);
        assert_eq!(a.number_arrivals(100), 10);
        assert_eq!(a.number_arrivals(101), 11);
        assert_eq!(a.number_arrivals(105), 11);
    }
    #[test]
    fn periodic_arrivals_unrolled() {
        let p = arrivals::Periodic{ period: 10 };
        let a = arrivals::CurvePrefix::from(p);
        assert_eq!(a.number_arrivals(0),  0);
        assert_eq!(a.number_arrivals(1),  1);
        assert_eq!(a.number_arrivals(8),  1);        
        assert_eq!(a.number_arrivals(10),  1);
        assert_eq!(a.number_arrivals(11),  2);
        assert_eq!(a.number_arrivals(12),  2);
        assert_eq!(a.number_arrivals(13),  2);
        assert_eq!(a.number_arrivals(100), 10);
        assert_eq!(a.number_arrivals(101), 11);
        assert_eq!(a.number_arrivals(105), 11);
    }

    #[test]
    fn periodic_arrivals_from_trace() {
        let trace: Vec<u64> = vec![0, 10, 20, 30, 40];
        let a = arrivals::CurvePrefix::from_trace(trace.iter(), 10);
        assert_eq!(a.number_arrivals(0),  0);
        assert_eq!(a.number_arrivals(1),  1);
        assert_eq!(a.number_arrivals(8),  1);        
        assert_eq!(a.number_arrivals(10),  1);
        assert_eq!(a.number_arrivals(11),  2);
        assert_eq!(a.number_arrivals(12),  2);
        assert_eq!(a.number_arrivals(13),  2);
        assert_eq!(a.number_arrivals(100), 10);
        assert_eq!(a.number_arrivals(101), 11);
        assert_eq!(a.number_arrivals(105), 11);
    }

    #[test]
    fn compare_periodic_arrivals() {
        let p = arrivals::Periodic{ period: 10 };
        let s = arrivals::Sporadic::from(p);
        let a = arrivals::CurvePrefix::from(p);
        let b = arrivals::CurvePrefix::from(s);
        let trace: Vec<u64> = vec![0, 10, 20, 30, 40];
        let t = arrivals::CurvePrefix::from_trace(trace.iter(), 2);
        for delta in 0..1000 {
            assert_eq!(a.number_arrivals(delta),  p.number_arrivals(delta));
            assert_eq!(s.number_arrivals(delta),  p.number_arrivals(delta));
            assert_eq!(a.number_arrivals(delta),  b.number_arrivals(delta));
            assert_eq!(p.number_arrivals(delta),  t.number_arrivals(delta));
        }
    }

    #[test]
    fn periodic_iter() {
        let p = arrivals::Periodic{ period: 10 };
        let steps: Vec<_> = p.steps_iter().take(5).collect();
        assert_eq!(steps, [1, 11, 21, 31, 41]);

        let p2 = arrivals::CurvePrefix::from(arrivals::Sporadic::from(p));
        let steps2: Vec<_> = p2.steps_iter().take(5).collect();
        assert_eq!(steps2, [1, 11, 21, 31, 41]);        
    }

    #[test]
    fn sporadic_arrivals() {
        let a = arrivals::Sporadic{ min_inter_arrival: 10, jitter: 3 };
        assert_eq!(a.number_arrivals(0),  0);
        assert_eq!(a.number_arrivals(1),  1);
        assert_eq!(a.number_arrivals(8),  2);
        assert_eq!(a.number_arrivals(10),  2);
        assert_eq!(a.number_arrivals(11),  2);
        assert_eq!(a.number_arrivals(100), 11);
        assert_eq!(a.number_arrivals(107), 11);
        assert_eq!(a.number_arrivals(108), 12);
        assert_eq!(a.number_arrivals(1108), 112);
    }

    #[test]
    fn sporadic_arrivals_from_trace() {
        let trace: Vec<u64> = vec![0, 7, 17, 27, 37, 47, 57, 67, 77, 87, 110, 117];
        let a = arrivals::CurvePrefix::from_trace(trace.iter(), 5);
        assert_eq!(a.number_arrivals(0),  0);
        assert_eq!(a.number_arrivals(1),  1);
        assert_eq!(a.number_arrivals(8),  2);
        assert_eq!(a.number_arrivals(10),  2);
        assert_eq!(a.number_arrivals(11),  2);
    }

    #[test]
    fn sporadic_arrivals_unrolled() {
        let s = arrivals::Sporadic{ min_inter_arrival: 10, jitter: 3 };
        let a = arrivals::CurvePrefix::from(s);
        assert_eq!(a.number_arrivals(0),  0);
        assert_eq!(a.number_arrivals(1),  1);
        assert_eq!(a.number_arrivals(8),  2);
        assert_eq!(a.number_arrivals(10),  2);
        assert_eq!(a.number_arrivals(11),  2);
        assert_eq!(a.number_arrivals(100), 11);
        assert_eq!(a.number_arrivals(107), 11);
        assert_eq!(a.number_arrivals(108), 12);
        assert_eq!(a.number_arrivals(1108), 112);
    }

    #[test]
    fn sporadic_arrivals_large_jitter() {
        let a = arrivals::Sporadic{ min_inter_arrival: 10, jitter: 16 };
        assert_eq!(a.number_arrivals(0),  0);
        assert_eq!(a.number_arrivals(1),  2);
        assert_eq!(a.number_arrivals(4),  2);
        assert_eq!(a.number_arrivals(5),  3);
    }

    #[test]
    fn compare_sporadic_arrivals() {
        let s = arrivals::Sporadic{ min_inter_arrival: 10, jitter: 16 };
        let a = arrivals::CurvePrefix::from(s);
        for delta in 0..1000 {
            assert_eq!(a.number_arrivals(delta),  s.number_arrivals(delta));
        }
    }

    #[test]
    fn sporadic_iter() {
        let s1 = arrivals::Sporadic{ min_inter_arrival: 10, jitter: 3 };
        let steps1: Vec<_> = s1.steps_iter().take(6).collect();
        assert_eq!(steps1, [1, 8, 18, 28, 38, 48]);

        let s2 = arrivals::Sporadic{ min_inter_arrival: 10, jitter: 16 };
        let steps2: Vec<_> = s2.steps_iter().take(6).collect();
        assert_eq!(steps2, [1, 5, 15, 25, 35, 45]);

        let s3 = arrivals::CurvePrefix::from(s1);
        let steps3: Vec<_> = s3.steps_iter().take(6).collect();
        assert_eq!(steps3, [1, 8, 18, 28, 38, 48]);

        let s4 = arrivals::CurvePrefix::from(s2);
        let steps4: Vec<_> = s4.steps_iter().take(6).collect();
        assert_eq!(steps4, [1, 5, 15, 25, 35, 45]);
    }

    #[test]
    fn poisson() {
        let p = arrivals::Poisson{ rate: 0.01 };
        assert_approx_eq!(p.arrival_probability(100, 0), 0.368, 0.001);
        assert_approx_eq!(p.arrival_probability(100, 1), 0.368, 0.001);
        assert_approx_eq!(p.arrival_probability(100, 2), 0.184, 0.001);
        assert_approx_eq!(p.arrival_probability(100, 3), 0.061, 0.001);
        assert_approx_eq!(p.arrival_probability(100, 4), 0.015, 0.001);
        assert_approx_eq!(p.arrival_probability(100, 5), 0.003, 0.001);
        assert_approx_eq!(p.arrival_probability(100, 6), 0.0005, 0.001);

        // let a = p.approximate(0.0001);

        // for y in a.steps_iter().take(100) {
        //     println!("step at interval length {} [#events <= {}]", y, a.number_arrivals(y));
        // }
        // let t = a.maximal_trace(2000);
        // for y in t {
        //     println!("event at time {} [at most {} events]", y, a.number_arrivals(y + 1));
        // }
    }

    #[test]
    fn propagated_jitter() {
        let p = arrivals::Periodic{ period: 10 };
        let s = arrivals::Sporadic{ min_inter_arrival: 10, jitter: 3};
        let prop = arrivals::Propagated{ input_event_model: p, response_time_jitter: 3};
        for t in 0..1000 {
            assert_eq!(s.number_arrivals(t), prop.number_arrivals(t));
        }
        for (x, y) in s.steps_iter().zip(prop.steps_iter().take(100)) {
            assert_eq!(x, y);
        }
    }

    #[test]
    fn periodic_supply() {
        let r = supply::Periodic{period: 5, budget: 3};

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

        assert_eq!(r.service_time(0),  0);
        assert_eq!(r.service_time(1),  5);
        assert_eq!(r.service_time(2),  6);
        assert_eq!(r.service_time(3),  7);
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
    }

    #[test]
    fn cost_models() {
        let wcet: Duration = 10;

        assert_eq!(wcet.cost_of_jobs(0),    0);
        assert_eq!(wcet.cost_of_jobs(3),   30);
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
    fn ros2_event_source() {
        let sbf = supply::Periodic{period: 5, budget: 3};

        let rbf = demand::RBF{
            wcet: 2,
            arrival_bound: arrivals::Sporadic{ jitter: 2, min_inter_arrival: 5 }
        };

        let result = ros2::rta_event_source(&sbf, &rbf, 100);

        assert!(result.is_some());
        assert_eq!(result.unwrap(), 7);
    }

    #[test]
    fn ros2_timer_periodic() {
        let sbf = supply::Periodic{period: 5, budget: 3};

        let rbf = demand::RBF{
            wcet: 1,
            arrival_bound: arrivals::Periodic{period: 10}
        };

        let interference = vec![
            demand::RBF{wcet: 1, arrival_bound: arrivals::Periodic{period: 10}},
            demand::RBF{wcet: 3, arrival_bound: arrivals::Periodic{period: 20}},
        ];

        let result = ros2::rta_timer(&sbf, &rbf, &interference, 0, 100);
        assert!(result.is_some());
        assert_eq!(result.unwrap(), 12);


        let result2 = ros2::rta_timer(&sbf, &rbf, &interference, 4, 100);
        assert!(result2.is_some());
        assert_eq!(result2.unwrap(), 20);
    }

    #[test]
    fn ros2_timer_sporadic() {
        let sbf = supply::Periodic{period: 5, budget: 3};

        let rbf = demand::RBF{
            wcet: 1,
            arrival_bound: arrivals::Periodic{period: 10}
        };

        let interference: Vec<Box<dyn RequestBound>> = vec![
            Box::new(demand::RBF{wcet: 1, arrival_bound: arrivals::Periodic{period: 10}}),
            Box::new(demand::RBF{wcet: 3, arrival_bound: arrivals::Sporadic{min_inter_arrival: 20, jitter: 10}}),
        ];

        let result = ros2::rta_timer(&sbf, &rbf, &interference, 0, 100);
        assert!(result.is_some());
        assert_eq!(result.unwrap(), 17);
    }

    #[test]
    fn ros2_pp_callback() {
        let sbf = supply::Periodic{period: 5, budget: 3};

        let rbf = demand::RBF{
            wcet: 1,
            arrival_bound: arrivals::Periodic{period: 10}
        };

        let interference = vec![
            demand::RBF{wcet: 1, arrival_bound: arrivals::Periodic{period: 10}},
            demand::RBF{wcet: 3, arrival_bound: arrivals::Periodic{period: 20}},
        ];

        let result = ros2::rta_polling_point_callback(&sbf, &rbf, &interference, 100);
        assert!(result.is_some());
        assert_eq!(result.unwrap(), 12);
    }

    #[test]
    fn ros2_chain() {
        let sbf = supply::Periodic{period: 5, budget: 3};

        let chain1_wcet = vec![1, 2, 3];
        let chain2_wcet = vec![1, 1, 1];
        let chain3_wcet = vec![2, 1];

        let total1: Duration = chain1_wcet.iter().sum();
        let total2: Duration = chain2_wcet.iter().sum();
        let total3: Duration = chain3_wcet.iter().sum();

        let all_chains: Vec<Box<dyn RequestBound>> = vec![
            Box::new(demand::RBF{
                wcet: total1,
                arrival_bound: arrivals::Periodic{period: 25}
            }),
            Box::new(demand::RBF{
                wcet: total2,
                arrival_bound: arrivals::Sporadic{min_inter_arrival: 20, jitter: 25}
            }),
            Box::new(demand::RBF{
                wcet: total3,
                arrival_bound: arrivals::Sporadic{min_inter_arrival: 20, jitter: 25}
            }),
        ];

        let last_cb = *chain1_wcet.iter().last().unwrap();

        let result = ros2::rta_processing_chain(&sbf, &all_chains, last_cb, 1000);
        assert!(result.is_some());
        assert_eq!(result.unwrap(), 152);
    }

    #[test]
    fn ros2_chain2() {
        let sbf = supply::Periodic{period: 5, budget: 3};

        let chain1_wcet = vec![1, 2, 3];
        let chain2_wcet = vec![1, 1, 1];
        let chain3_wcet = vec![2, 1];

        let prefix1: Duration = chain1_wcet[0..2].iter().sum();
        let total2: Duration = chain2_wcet.iter().sum();
        let total3: Duration = chain3_wcet.iter().sum();

        let chain1_prefix = demand::RBF{
                wcet: prefix1,
                arrival_bound: arrivals::Periodic{period: 25}
        };

        let chain1_suffix = demand::RBF{
                wcet: chain1_wcet[2],
                arrival_bound: arrivals::Periodic{period: 25}
        };

        let other_chains: Vec<Box<dyn RequestBound>> = vec![
            Box::new(demand::RBF{
                wcet: total2,
                arrival_bound: arrivals::Sporadic{min_inter_arrival: 20, jitter: 25}
            }),
            Box::new(demand::RBF{
                wcet: total3,
                arrival_bound: arrivals::Sporadic{min_inter_arrival: 20, jitter: 25}
            }),
        ];

        let result = ros2::rta_processing_chain2(&sbf, &chain1_prefix, &chain1_suffix, &other_chains, 1000);
        assert!(result.is_some());
        dbg!(result);
        assert_eq!(result.unwrap(), 72);
    }

    #[test]
    fn ros2_chain3() {
        let sbf = supply::Periodic{period: 5, budget: 3};

        let chain1_wcet: Vec<Duration> = vec![1, 2, 3];
        let chain2_wcet = vec![1, 1, 1];
        let chain3_wcet = vec![2, 1];

        let total2: Duration = chain2_wcet.iter().sum();
        let total3: Duration = chain3_wcet.iter().sum();

        let chain1_arrivals = arrivals::Periodic{period: 25};

        let other_chains: Vec<Box<dyn RequestBound>> = vec![
            Box::new(demand::RBF{
                wcet: total2,
                arrival_bound: arrivals::Sporadic{min_inter_arrival: 20, jitter: 25}
            }),
            Box::new(demand::RBF{
                wcet: total3,
                arrival_bound: arrivals::Sporadic{min_inter_arrival: 20, jitter: 25}
            }),
        ];

        let result = ros2::rta_processing_chain_window_aware(
            &sbf,
            chain1_wcet.iter().copied(),
            &chain1_arrivals,
            &other_chains,
            1000
        );
        assert!(result.is_some());
        dbg!(result);
        // assert_eq!(result.unwrap(), 72);
    }
}
