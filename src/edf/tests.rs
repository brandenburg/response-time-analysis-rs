use crate::{
    arrival::Sporadic,
    demand, edf,
    tests::{d, s},
    time::Duration,
    wcet,
};

#[test]
fn edf_rta_nonpreemptive_same_deadlines() {
    let horizon = d(1000);
    let params = vec![(79, 120), (11, 34), (1, 190)];
    let deadlines = vec![
        Duration::from(100),
        Duration::from(100),
        Duration::from(100),
    ];

    let expected = vec![Some(91), Some(91), Some(91)];

    let arrivals: Vec<Sporadic> = params
        .iter()
        .map(|(_wcet, inter_arrival)| Sporadic::new_zero_jitter(d(*inter_arrival)))
        .collect();

    let costs: Vec<wcet::Scalar> = params
        .iter()
        .map(|(wcet, _)| wcet::Scalar::new(s(*wcet)))
        .collect();

    for (i, expected_bound) in expected.iter().enumerate() {
        let tua = edf::fully_nonpreemptive::TaskUnderAnalysis {
            wcet: costs[i],
            arrivals: &arrivals[i],
            deadline: deadlines[i],
        };
        let other_tasks: Vec<edf::fully_nonpreemptive::InterferingTask<_>> = (0..expected.len())
            .filter(|j| *j != i)
            .map(|j| edf::fully_nonpreemptive::InterferingTask {
                wcet: costs[j],
                arrivals: &arrivals[j],
                deadline: deadlines[j],
            })
            .collect();

        let result = edf::fully_nonpreemptive::dedicated_uniproc_rta(&tua, &other_tasks, horizon);

        assert_eq!(expected_bound.map(d), result.ok());
    }
}

#[test]
fn edf_rta_nonpreemptive_different_deadlines() {
    let horizon = d(1000);
    let params = vec![(79, 120), (11, 34), (1, 190)];
    let deadlines = vec![Duration::from(50), Duration::from(100), Duration::from(120)];

    let expected = vec![Some(89), Some(90), Some(121)];

    let arrivals: Vec<Sporadic> = params
        .iter()
        .map(|(_wcet, inter_arrival)| Sporadic::new_zero_jitter(d(*inter_arrival)))
        .collect();

    let costs: Vec<wcet::Scalar> = params
        .iter()
        .map(|(wcet, _)| wcet::Scalar::new(s(*wcet)))
        .collect();

    for (i, expected_bound) in expected.iter().enumerate() {
        let tua = edf::fully_nonpreemptive::TaskUnderAnalysis {
            wcet: costs[i],
            arrivals: &arrivals[i],
            deadline: deadlines[i],
        };
        let other_tasks: Vec<edf::fully_nonpreemptive::InterferingTask<_>> = (0..expected.len())
            .filter(|j| *j != i)
            .map(|j| edf::fully_nonpreemptive::InterferingTask {
                wcet: costs[j],
                arrivals: &arrivals[j],
                deadline: deadlines[j],
            })
            .collect();

        let result = edf::fully_nonpreemptive::dedicated_uniproc_rta(&tua, &other_tasks, horizon);
        assert_eq!(expected_bound.map(d), result.ok());
    }
}

#[test]
fn edf_rta_nonpreemptive_improved_bound() {
    let horizon = d(1000);
    let params = vec![(5, 20), (10, 20)];
    let deadlines = vec![Duration::from(29), Duration::from(30)];

    let expected = vec![Some(14), Some(15)];

    let arrivals: Vec<Sporadic> = params
        .iter()
        .map(|(_wcet, inter_arrival)| Sporadic::new_zero_jitter(d(*inter_arrival)))
        .collect();

    let costs: Vec<wcet::Scalar> = params
        .iter()
        .map(|(wcet, _)| wcet::Scalar::new(s(*wcet)))
        .collect();

    for (i, expected_bound) in expected.iter().enumerate() {
        let tua = edf::fully_nonpreemptive::TaskUnderAnalysis {
            wcet: costs[i],
            arrivals: &arrivals[i],
            deadline: deadlines[i],
        };
        let other_tasks: Vec<edf::fully_nonpreemptive::InterferingTask<_>> = (0..expected.len())
            .filter(|j| *j != i)
            .map(|j| edf::fully_nonpreemptive::InterferingTask {
                wcet: costs[j],
                arrivals: &arrivals[j],
                deadline: deadlines[j],
            })
            .collect();

        let result = edf::fully_nonpreemptive::dedicated_uniproc_rta(&tua, &other_tasks, horizon);
        assert_eq!(expected_bound.map(d), result.ok());
    }
}

#[test]
fn edf_rta_fully_preemptive_ex1() {
    let horizon = d(100000);
    let params = vec![(79, 120), (11, 34), (1, 190)];
    let deadlines = vec![Duration::from(50), Duration::from(100), Duration::from(120)];

    let expected = vec![Some(79), Some(101), Some(121)];

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
        let tua = edf::fully_preemptive::TaskUnderAnalysis {
            rbf: &rbfs[i],
            deadline: deadlines[i],
        };
        let other_tasks: Vec<edf::fully_preemptive::InterferingTask<_>> = (0..expected.len())
            .filter(|j| *j != i)
            .map(|j| edf::fully_preemptive::InterferingTask {
                rbf: &rbfs[j],
                deadline: deadlines[j],
            })
            .collect();

        let result = edf::fully_preemptive::dedicated_uniproc_rta(&tua, &other_tasks, horizon);
        assert_eq!(expected_bound.map(d), result.ok());
    }
}

#[test]
fn edf_rta_fully_preemptive_ex2() {
    let horizon = d(10000);
    let params = vec![(1, 5), (100, 1000), (2, 10), (5, 20), (10, 50)];
    let deadlines = vec![
        Duration::from(5),
        Duration::from(1000),
        Duration::from(10),
        Duration::from(45),
        Duration::from(50),
    ];

    let expected = vec![Some(1), Some(694), Some(3), Some(22), Some(27)];

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
        let tua = edf::fully_preemptive::TaskUnderAnalysis {
            rbf: &rbfs[i],
            deadline: deadlines[i],
        };
        let other_tasks: Vec<edf::fully_preemptive::InterferingTask<_>> = (0..expected.len())
            .filter(|j| *j != i)
            .map(|j| edf::fully_preemptive::InterferingTask {
                rbf: &rbfs[j],
                deadline: deadlines[j],
            })
            .collect();

        let result = edf::fully_preemptive::dedicated_uniproc_rta(&tua, &other_tasks, horizon);
        assert_eq!(expected_bound.map(d), result.ok());
    }
}

#[test]
fn edf_rta_floating_nonpreemptive_ex1() {
    let horizon = d(100);
    let params = vec![(4, 12), (6, 20), (8, 40)];
    let deadlines = vec![Duration::from(25), Duration::from(30), Duration::from(40)];
    let max_non_preemptive_sections = vec![s(2), s(3), s(4)];

    let expected = vec![Some(8), Some(13), Some(22)];

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
        let tua = edf::floating_nonpreemptive::TaskUnderAnalysis {
            rbf: &rbfs[i],
            deadline: deadlines[i],
        };
        let other_tasks: Vec<edf::floating_nonpreemptive::InterferingTask<_>> = (0..expected.len())
            .filter(|j| *j != i)
            .map(|j| edf::floating_nonpreemptive::InterferingTask {
                deadline: deadlines[j],
                rbf: &rbfs[j],
                max_np_segment: max_non_preemptive_sections[j],
            })
            .collect();

        let result =
            edf::floating_nonpreemptive::dedicated_uniproc_rta(&tua, &other_tasks, horizon);
        assert_eq!(expected_bound.map(d), result.ok());
    }
}

#[test]
fn edf_rta_limited_preemptive_ex1() {
    let horizon = d(1000);
    let params = vec![(4, 12), (6, 20), (8, 40)];
    let deadlines = vec![Duration::from(24), Duration::from(35), Duration::from(40)];
    let max_non_preemptive_sections = vec![s(2), s(3), s(4)];
    let last_non_preemptive_sections = vec![s(2), s(3), s(4)];
    let expected = vec![Some(7), Some(17), Some(22)];

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
        let tua = edf::limited_preemptive::TaskUnderAnalysis {
            wcet: costs[i],
            arrivals: &arrivals[i],
            deadline: deadlines[i],
            last_np_segment: last_non_preemptive_sections[i],
        };
        let other_tasks: Vec<edf::limited_preemptive::InterferingTask<_>> = (0..expected.len())
            .filter(|j| *j != i)
            .map(|j| edf::limited_preemptive::InterferingTask {
                deadline: deadlines[j],
                rbf: &rbfs[j],
                max_np_segment: max_non_preemptive_sections[j],
            })
            .collect();

        let result = edf::limited_preemptive::dedicated_uniproc_rta(&tua, &other_tasks, horizon);
        assert_eq!(expected_bound.map(d), result.ok());
    }
}
