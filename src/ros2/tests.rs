use crate::arrival::ArrivalBound;
use crate::demand::RequestBound;
use crate::supply::SupplyBound;
use crate::time::Duration;
use crate::wcet::JobCostModel;
use crate::{arrival, demand, ros2, supply, wcet};

use crate::tests::{d, s};

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
    assert_eq!(result, Ok(d(0)));
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
    assert_eq!(result, Ok(d(12)));

    let result2 = ros2::rta_timer(&sbf, &rbf, &interference, s(4), d(100));
    assert_eq!(result2, Ok(d(20)));
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

    let b0 =
        ros2::rr::rta_subchain(&sbf, workload, &sc0[..], d(100000)).expect("no fixed point found");
    let b1 =
        ros2::rr::rta_subchain(&sbf, workload, &sc1[..], d(100000)).expect("no fixed point found");
    let b2 =
        ros2::rr::rta_subchain(&sbf, workload, &sc2[..], d(100000)).expect("no fixed point found");

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

    let b0 =
        ros2::bw::rta_subchain(&sbf, workload, &sc0[..], d(100000)).expect("no fixed point found");
    let b1 =
        ros2::bw::rta_subchain(&sbf, workload, &sc1[..], d(100000)).expect("no fixed point found");
    let b2 =
        ros2::bw::rta_subchain(&sbf, workload, &sc2[..], d(100000)).expect("no fixed point found");

    assert_eq!(b0, assumed_response_time_bounds[0]);
    assert_eq!(b1, assumed_response_time_bounds[1]);
    assert_eq!(b2, assumed_response_time_bounds[2]);
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

    let res =
        ros2::rr::rta_subchain(&sbf, workload, &sc[..], d(100000)).expect("no fixed point found");
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

    let res =
        ros2::bw::rta_subchain(&sbf, workload, &sc[..], d(100000)).expect("no fixed point found");
    assert_eq!(res, d(5));
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
