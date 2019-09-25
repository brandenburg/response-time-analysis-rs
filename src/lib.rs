pub mod arrivals;

#[cfg(test)]
mod tests {
    use crate::arrivals::{self, ArrivalBound};
    use assert_approx_eq::assert_approx_eq;

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
    fn poisson() {
        let p = arrivals::Poisson{ rate: 0.01 };
        assert_approx_eq!(p.arrival_probability(100, 0), 0.368, 0.001);
        assert_approx_eq!(p.arrival_probability(100, 1), 0.368, 0.001);
        assert_approx_eq!(p.arrival_probability(100, 2), 0.184, 0.001);
        assert_approx_eq!(p.arrival_probability(100, 3), 0.061, 0.001);
        assert_approx_eq!(p.arrival_probability(100, 4), 0.015, 0.001);
        assert_approx_eq!(p.arrival_probability(100, 5), 0.003, 0.001);
        assert_approx_eq!(p.arrival_probability(100, 6), 0.0005, 0.001);

        let a = p.approximate(0.0001);
        let t = a.maximal_trace(2000);
        for y in t {
            println!("flood at {} [at most {} floods]", y, a.number_arrivals(y + 1));
        }
    }
    
}
