use crate::arrival::Sporadic;
use crate::demand;
use crate::fixed_priority;
use crate::time::Service;
use crate::wcet;

use crate::tests::{d, s};

#[test]
fn fp_fp_rta_basic() {
    let horizon = d(100);
    let params = vec![(1, 4), (1, 5), (3, 9), (3, 18)];
    let expected = vec![1, 2, 7, 18];

    let arrivals: Vec<Sporadic> = params
        .iter()
        .map(|(_wcet, inter_arrival)| Sporadic::new_zero_jitter(d(*inter_arrival)))
        .collect();

    let costs: Vec<wcet::Scalar> = params
        .iter()
        .map(|(wcet, _)| wcet::Scalar::new(s(*wcet)))
        .collect();

    let rbfs: Vec<demand::RBF<_, _>> = costs
        .iter()
        .zip(arrivals.iter())
        .map(|(wcet, arr)| demand::RBF::new(arr, wcet))
        .collect();

    for (i, expected_bound) in expected.iter().enumerate() {
        let interference = demand::Slice::of(&rbfs[0..i]);
        let result = fixed_priority::fully_preemptive::dedicated_uniproc_rta(
            &interference,
            &costs[i],
            &arrivals[i],
            horizon,
        );
        assert_eq!(Ok(d(*expected_bound)), result);
    }
}

/// Example 2 in "Fixed Priority Scheduling of Periodic Task Sets
/// with Arbitrary Deadlines", John P. Lehoczky, RTSS 1990.
#[test]
fn fp_fp_rta_lehoczky90_ex2() {
    let horizon = d(1000);
    let params = vec![(52, 100), (52, 140)];
    let expected1 = vec![52, 156]; // first task has higher prio
    let expected2 = vec![108, 52]; // second task has higher prio

    let arrivals: Vec<Sporadic> = params
        .iter()
        .map(|(_wcet, inter_arrival)| Sporadic::new_zero_jitter(d(*inter_arrival)))
        .collect();

    let costs: Vec<wcet::Scalar> = params
        .iter()
        .map(|(wcet, _)| wcet::Scalar::new(s(*wcet)))
        .collect();

    let rbfs: Vec<demand::RBF<_, _>> = costs
        .iter()
        .zip(arrivals.iter())
        .map(|(wcet, arr)| demand::RBF::new(arr, wcet))
        .collect();

    for (i, expected_bound) in expected1.iter().enumerate() {
        // lower-indexed task == higher-priority task
        let interference = demand::Slice::of(&rbfs[0..i]);
        let result = fixed_priority::fully_preemptive::dedicated_uniproc_rta(
            &interference,
            &costs[i],
            &arrivals[i],
            horizon,
        );
        assert_eq!(Ok(d(*expected_bound)), result);
    }

    for (i, expected_bound) in expected2.iter().enumerate() {
        // higher-indexed task == higher-priority task
        let interference = demand::Slice::of(&rbfs[i + 1..]);
        let result = fixed_priority::fully_preemptive::dedicated_uniproc_rta(
            &interference,
            &costs[i],
            &arrivals[i],
            horizon,
        );
        assert_eq!(Ok(d(*expected_bound)), result);
    }
}

/// Example 3 in "Fixed Priority Scheduling of Periodic Task Sets
/// with Arbitrary Deadlines", John P. Lehoczky, RTSS 1990.
#[test]
fn fp_fp_rta_lehoczky90_ex3() {
    let horizon = d(1000);
    let params = vec![(26, 70), (62, 100)];
    let expected = vec![26, 118];

    let arrivals: Vec<Sporadic> = params
        .iter()
        .map(|(_wcet, inter_arrival)| Sporadic::new_zero_jitter(d(*inter_arrival)))
        .collect();

    let costs: Vec<wcet::Scalar> = params
        .iter()
        .map(|(wcet, _)| wcet::Scalar::new(s(*wcet)))
        .collect();

    let rbfs: Vec<demand::RBF<_, _>> = costs
        .iter()
        .zip(arrivals.iter())
        .map(|(wcet, arr)| demand::RBF::new(arr, wcet))
        .collect();

    for (i, expected_bound) in expected.iter().enumerate() {
        // lower-indexed task == higher-priority task
        let interference = demand::Slice::of(&rbfs[0..i]);
        let result = fixed_priority::fully_preemptive::dedicated_uniproc_rta(
            &interference,
            &costs[i],
            &arrivals[i],
            horizon,
        );
        assert_eq!(Ok(d(*expected_bound)), result);
    }
}

/// Reference task sets and bounds from SchedCAT
#[test]
fn fp_fp_rta_schedcat() {
    let horizon = d(200000);
    let tasksets = vec![
        vec![
            (5995, 43000, 5995),
            (2497, 44000, 8492),
            (18376, 52000, 26868),
            (7724, 55000, 34592),
        ],
        vec![
            (2693, 84000, 2693),
            (1340, 92000, 4033),
            (462, 34000, 4495),
            (438, 39000, 4933),
            (1350, 32000, 6283),
            (1080, 12000, 7363),
            (6083, 91000, 14526),
            (550, 57000, 15076),
            (3486, 68000, 18562),
            (4916, 81000, 23478),
        ],
        vec![
            (1274, 13000, 1274),
            (5840, 16000, 7114),
            (1433, 15000, 8547),
            (1579, 13000, 10126),
        ],
        vec![
            (3691, 121000, 3691),
            (2301, 57000, 5992),
            (1493, 224000, 7485),
            (573, 72000, 8058),
            (13152, 53000, 21210),
            (951, 65000, 22161),
        ],
        vec![
            (108, 6000, 108),
            (1202, 13000, 1310),
            (8280, 30000, 9698),
            (7904, 30000, 19020),
            (569, 28000, 19589),
            (117, 22000, 19706),
            (1288, 29000, 20994),
        ],
        vec![
            (931, 224000, 931),
            (6600, 118000, 7531),
            (10729, 117000, 18260),
            (2202, 153000, 20462),
            (5174, 138000, 25636),
        ],
        vec![
            (1064, 12000, 1064),
            (409, 13000, 1473),
            (1671, 41000, 3144),
            (2341, 33000, 5485),
            (512, 31000, 5997),
            (133, 31000, 6130),
            (2387, 32000, 8517),
            (5518, 63000, 15508),
            (1391, 24000, 16899),
        ],
        vec![
            (828, 17000, 828),
            (936, 24000, 1764),
            (1141, 59000, 2905),
            (1036, 67000, 3941),
            (468, 14000, 4409),
            (444, 19000, 4853),
            (2477, 41000, 7330),
            (367, 20000, 7697),
            (4382, 69000, 12079),
            (2008, 23000, 14555),
        ],
        vec![
            (9214, 99000, 9214),
            (1646, 19000, 10860),
            (901, 17000, 11761),
            (226, 35000, 11987),
            (2101, 62000, 14088),
            (6606, 82000, 23241),
            (1353, 35000, 24594),
            (875, 42000, 25469),
        ],
        vec![
            (172, 8000, 172),
            (280, 32000, 452),
            (1073, 18000, 1525),
            (68, 22000, 1593),
            (658, 20000, 2251),
            (501, 32000, 2752),
            (2587, 33000, 5339),
            (1133, 20000, 6472),
        ],
        vec![
            (10335, 173000, 10335),
            (26798, 80000, 37133),
            (9025, 93000, 46158),
            (3586, 89000, 49744),
            (1760, 94000, 51504),
            (496, 57000, 52000),
            (3014, 89000, 55014),
            (8509, 111000, 64019),
        ],
        vec![
            (2914, 227000, 2914),
            (8598, 96000, 11512),
            (2085, 60000, 13597),
            (1858, 233000, 15455),
            (4498, 86000, 19953),
            (1869, 58000, 21822),
            (2906, 149000, 24728),
            (6661, 246000, 31389),
        ],
        vec![
            (4266, 52000, 4266),
            (3046, 53000, 7312),
            (4921, 66000, 12233),
            (2597, 102000, 14830),
            (7535, 84000, 22365),
        ],
        vec![
            (17116, 50000, 17116),
            (7200, 100000, 24316),
            (14384, 138000, 38700),
            (13596, 122000, 69412),
        ],
        vec![
            (36474, 66000, 36474),
            (4921, 95000, 41395),
            (984, 95000, 42379),
            (18668, 113000, 61047),
        ],
    ];

    for ts in &tasksets {
        let arrivals: Vec<Sporadic> = ts
            .iter()
            .map(|(_wcet, inter_arrival, _response_time)| {
                Sporadic::new_zero_jitter(d(*inter_arrival))
            })
            .collect();

        let costs: Vec<wcet::Scalar> = ts
            .iter()
            .map(|(wcet, _, _)| wcet::Scalar::new(s(*wcet)))
            .collect();

        let rbfs: Vec<demand::RBF<_, _>> = costs
            .iter()
            .zip(arrivals.iter())
            .map(|(wcet, arr)| demand::RBF::new(arr, wcet))
            .collect();

        for (i, expected_bound) in ts.iter().map(|(_, _, rt)| rt).enumerate() {
            let interference = demand::Slice::of(&rbfs[0..i]);
            let result = fixed_priority::fully_preemptive::dedicated_uniproc_rta(
                &interference,
                &costs[i],
                &arrivals[i],
                horizon,
            );
            assert_eq!(Ok(d(*expected_bound)), result);
        }
    }
}

#[test]
fn fp_fp_rta_overload() {
    let horizon = d(100);
    let params = vec![(1, 2), (1, 3), (3, 9), (3, 18)];
    let expected = vec![Some(1), Some(2), None, None];

    let arrivals: Vec<Sporadic> = params
        .iter()
        .map(|(_wcet, inter_arrival)| Sporadic::new_zero_jitter(d(*inter_arrival)))
        .collect();

    let costs: Vec<wcet::Scalar> = params
        .iter()
        .map(|(wcet, _)| wcet::Scalar::new(s(*wcet)))
        .collect();

    let rbfs: Vec<demand::RBF<_, _>> = costs
        .iter()
        .zip(arrivals.iter())
        .map(|(wcet, arr)| demand::RBF::new(arr, wcet))
        .collect();

    for (i, expected_bound) in expected.iter().enumerate() {
        let interference = demand::Slice::of(&rbfs[0..i]);
        let result = fixed_priority::fully_preemptive::dedicated_uniproc_rta(
            &interference,
            &costs[i],
            &arrivals[i],
            horizon,
        );
        assert_eq!(expected_bound.map(d), result.ok());
    }
}

#[test]
fn fp_np_rta_1() {
    let horizon = d(1000);
    let params = vec![(20, 70), (20, 80), (35, 200)];

    let expected = vec![54, 74, 75];

    let arrivals: Vec<Sporadic> = params
        .iter()
        .map(|(_wcet, inter_arrival)| Sporadic::new_zero_jitter(d(*inter_arrival)))
        .collect();

    let costs: Vec<wcet::Scalar> = params
        .iter()
        .map(|(wcet, _)| wcet::Scalar::new(s(*wcet)))
        .collect();

    let rbfs: Vec<demand::RBF<_, _>> = costs
        .iter()
        .zip(arrivals.iter())
        .map(|(wcet, arr)| demand::RBF::new(arr, wcet))
        .collect();

    for (i, expected_bound) in expected.iter().enumerate() {
        let interference = demand::Slice::of(&rbfs[0..i]);
        let blocking_bound = rbfs[i + 1..]
            .iter()
            .map(|r| r.wcet.wcet)
            .max_by_key(|wcet| *wcet)
            .unwrap_or(s(0));

        let result = fixed_priority::fully_nonpreemptive::dedicated_uniproc_rta(
            &interference,
            &costs[i],
            &arrivals[i],
            blocking_bound.saturating_sub(s(1)),
            horizon,
        );
        assert_eq!(Ok(d(*expected_bound)), result);
    }
}

#[test]
fn fp_np_rta_overload() {
    let horizon = d(10000);
    let params = vec![(10, 20), (20, 50), (30, 200)];

    let expected = vec![Some(39), Some(79), None];

    let arrivals: Vec<Sporadic> = params
        .iter()
        .map(|(_wcet, inter_arrival)| Sporadic::new_zero_jitter(d(*inter_arrival)))
        .collect();

    let costs: Vec<wcet::Scalar> = params
        .iter()
        .map(|(wcet, _)| wcet::Scalar::new(s(*wcet)))
        .collect();

    let rbfs: Vec<demand::RBF<_, _>> = costs
        .iter()
        .zip(arrivals.iter())
        .map(|(wcet, arr)| demand::RBF::new(arr, wcet))
        .collect();

    for (i, expected_bound) in expected.iter().enumerate() {
        let interference = demand::Slice::of(&rbfs[0..i]);
        let blocking_bound = rbfs[i + 1..]
            .iter()
            .map(|r| r.wcet.wcet)
            .max_by_key(|wcet| *wcet)
            .unwrap_or(s(0));

        let result = fixed_priority::fully_nonpreemptive::dedicated_uniproc_rta(
            &interference,
            &costs[i],
            &arrivals[i],
            blocking_bound.saturating_sub(s(1)),
            horizon,
        );
        assert_eq!(expected_bound.map(d), result.ok());
    }
}

#[test]
fn fp_lp_rta_1() {
    let horizon = d(100);
    let params = vec![(4, 12), (6, 20), (8, 40)];
    let max_nonpr_segment: Vec<u64> = vec![2, 3, 4];
    let task_last_nonpr_segment: Vec<u64> = vec![2, 3, 3];

    let expected: Vec<u64> = vec![7, 13, 22];

    let arrivals: Vec<Sporadic> = params
        .iter()
        .map(|(_wcet, inter_arrival)| Sporadic::new_zero_jitter(d(*inter_arrival)))
        .collect();

    let costs: Vec<wcet::Scalar> = params
        .iter()
        .map(|(wcet, _)| wcet::Scalar::new(s(*wcet)))
        .collect();

    let rbfs: Vec<demand::RBF<_, _>> = costs
        .iter()
        .zip(arrivals.iter())
        .map(|(wcet, arr)| demand::RBF::new(arr, wcet))
        .collect();

    let task_last_segment: Vec<Service> = task_last_nonpr_segment
        .iter()
        .map(|m| Service::from(*m))
        .collect();

    for (i, expected_bound) in expected.iter().enumerate() {
        let interference = demand::Slice::of(&rbfs[0..i]);
        let blocking_bound = max_nonpr_segment[i + 1..].iter().max().unwrap_or(&0);

        let result = fixed_priority::limited_preemptive::dedicated_uniproc_rta(
            &interference,
            &costs[i],
            &arrivals[i],
            task_last_segment[i],
            s(*blocking_bound).saturating_sub(s(1)),
            horizon,
        );
        assert_eq!(Ok(d(*expected_bound)), result);
    }
}

#[test]
fn fp_lp_rta_overload() {
    let horizon = d(100000);
    let params = vec![(4, 12), (6, 20), (8, 21)];
    let max_nonpr_segment: Vec<u64> = vec![2, 3, 4];
    let task_last_nonpr_segment: Vec<u64> = vec![2, 3, 3];

    let expected = vec![Some(7), Some(13), None];

    let arrivals: Vec<Sporadic> = params
        .iter()
        .map(|(_wcet, inter_arrival)| Sporadic::new_zero_jitter(d(*inter_arrival)))
        .collect();

    let costs: Vec<wcet::Scalar> = params
        .iter()
        .map(|(wcet, _)| wcet::Scalar::new(s(*wcet)))
        .collect();

    let rbfs: Vec<demand::RBF<_, _>> = costs
        .iter()
        .zip(arrivals.iter())
        .map(|(wcet, arr)| demand::RBF::new(arr, wcet))
        .collect();

    let task_last_segment: Vec<Service> = task_last_nonpr_segment
        .iter()
        .map(|m| Service::from(*m))
        .collect();

    for (i, expected_bound) in expected.iter().enumerate() {
        let interference = demand::Slice::of(&rbfs[0..i]);
        let blocking_bound = max_nonpr_segment[i + 1..].iter().max().unwrap_or(&0);

        let result = fixed_priority::limited_preemptive::dedicated_uniproc_rta(
            &interference,
            &costs[i],
            &arrivals[i],
            task_last_segment[i],
            s(*blocking_bound).saturating_sub(s(1)),
            horizon,
        );
        assert_eq!(expected_bound.map(d), result.ok());
    }
}

#[test]
fn fp_fnps_rta_1() {
    let horizon = d(100);
    let params = vec![(4, 12), (6, 20), (8, 40)];
    let max_nonpr_segment: Vec<u64> = vec![2, 3, 4];

    let expected: Vec<u64> = vec![7, 17, 32];

    let arrivals: Vec<Sporadic> = params
        .iter()
        .map(|(_wcet, inter_arrival)| Sporadic::new_zero_jitter(d(*inter_arrival)))
        .collect();

    let costs: Vec<wcet::Scalar> = params
        .iter()
        .map(|(wcet, _)| wcet::Scalar::new(s(*wcet)))
        .collect();

    let rbfs: Vec<demand::RBF<_, _>> = costs
        .iter()
        .zip(arrivals.iter())
        .map(|(wcet, arr)| demand::RBF::new(arr, wcet))
        .collect();

    for (i, expected_bound) in expected.iter().enumerate() {
        let interference = demand::Slice::of(&rbfs[0..i]);
        let blocking_bound = max_nonpr_segment[i + 1..].iter().max().unwrap_or(&0);

        let result = fixed_priority::floating_nonpreemptive::dedicated_uniproc_rta(
            &interference,
            &costs[i],
            &arrivals[i],
            s(*blocking_bound).saturating_sub(s(1)),
            horizon,
        );
        assert_eq!(Ok(d(*expected_bound)), result);
    }
}

#[test]
fn fp_fnps_rta_overload() {
    let horizon = d(10000);
    let params = vec![(4, 12), (6, 20), (8, 30), (8, 40)];
    let max_nonpr_segment: Vec<u64> = vec![2, 3, 3, 4];

    let expected = vec![Some(7), Some(17), Some(35), None];

    let arrivals: Vec<Sporadic> = params
        .iter()
        .map(|(_wcet, inter_arrival)| Sporadic::new_zero_jitter(d(*inter_arrival)))
        .collect();

    let costs: Vec<wcet::Scalar> = params
        .iter()
        .map(|(wcet, _)| wcet::Scalar::new(s(*wcet)))
        .collect();

    let rbfs: Vec<demand::RBF<_, _>> = costs
        .iter()
        .zip(arrivals.iter())
        .map(|(wcet, arr)| demand::RBF::new(arr, wcet))
        .collect();

    for (i, expected_bound) in expected.iter().enumerate() {
        let interference = demand::Slice::of(&rbfs[0..i]);
        let blocking_bound = max_nonpr_segment[i + 1..].iter().max().unwrap_or(&0);

        let result = fixed_priority::floating_nonpreemptive::dedicated_uniproc_rta(
            &interference,
            &costs[i],
            &arrivals[i],
            s(*blocking_bound).saturating_sub(s(1)),
            horizon,
        );
        assert_eq!(expected_bound.map(d), result.ok());
    }
}

/// The Example in Table 3 of "Applying new sc;hedulingtheory to
/// static priority pre-emptive scheduling", Audsley et al., Software
/// Engineering Journal, 1993.
#[test]
fn fp_fnps_rta_audsley() {
    let horizon = d(1000000);
    // (cost, period, deadline, max non-preemptive segment, jitter, expected bound)
    let example_tasks = vec![
        (51, 1000, 1000, 0, 0, 51),
        (3000, 2000000, 5000, 300, 0, 3504),
        (2000, 25000, 25000, 600, 0, 5906),
        (5000, 25000, 25000, 900, 0, 11512),
        (1000, 40000, 40000, 1350, 0, 13064),
        (3000, 50000, 50000, 1350, 0, 16217),
        (5000, 50000, 50000, 750, 0, 20821),
        (8000, 59000, 59000, 750, 0, 36637),
        (9000, 80000, 80000, 1350, 0, 47798),
        (2000, 80000, 80000, 450, 0, 48949),
        (5000, 100000, 100000, 1050, 0, 99150),
        (1000, 200000, 200000, 450, 1000, 99550),
        (3000, 200000, 200000, 450, 0, 140641),
        (1000, 200000, 200000, 450, 0, 141692),
        (1000, 200000, 200000, 1350, 0, 143694),
        (3000, 1000000, 1000000, 0, 0, 145446),
        (1000, 1000000, 1000000, 0, 0, 146497),
        (1000, 1000000, 1000000, 0, 0, 147548),
    ];

    let arrivals: Vec<Sporadic> = example_tasks
        .iter()
        .map(|(_cost, period, _deadline, _nps, jitter, _expected)| {
            Sporadic::new(d(*period), d(*jitter))
        })
        .collect();
    let costs: Vec<wcet::Scalar> = example_tasks
        .iter()
        .map(|(cost, _period, _deadline, _nps, _jitter, _expected)| wcet::Scalar::new(s(*cost)))
        .collect();
    let rbfs: Vec<demand::RBF<_, _>> = costs
        .iter()
        .zip(arrivals.iter())
        .map(|(wcet, arr)| demand::RBF::new(arr, wcet))
        .collect();

    for (i, (_cost, _period, _deadline, blocking_bound, _jitter, expected)) in
        example_tasks.iter().enumerate()
    {
        let interference = demand::Slice::of(&rbfs[0..i]);

        let result = fixed_priority::floating_nonpreemptive::dedicated_uniproc_rta(
            &interference,
            &costs[i],
            &arrivals[i],
            s(*blocking_bound),
            horizon,
        );
        assert_eq!(Ok(d(*expected)), result);
    }
}
