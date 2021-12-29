use crate::supply::{self, SupplyBound};
use crate::tests::{d, s};
use crate::time::Duration;

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
        let blackout_interference = service_time - Duration::from(r.provided_service(service_time));
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
