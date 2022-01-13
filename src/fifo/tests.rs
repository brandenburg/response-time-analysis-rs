use crate::{
    arrival::Sporadic,
    demand, fifo,
    tests::{d, s},
    wcet,
};

#[test]
fn fifo_rta_ex1() {
    let horizon = d(1000);
    let params = vec![(79, 120), (11, 34), (1, 190)];

    let expected = Some(91);

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

    let interference = demand::Slice::of(&rbfs);
    let result = fifo::dedicated_uniproc_rta(&interference, horizon);
    assert_eq!(expected.map(d), result.ok());
}

#[test]
fn fifo_rta_ex2() {
    let horizon = d(1000);
    let params = vec![(2, 4), (2, 8), (4, 12)];

    let expected = None;

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

    let interference = demand::Slice::of(&rbfs);
    let result = fifo::dedicated_uniproc_rta(&interference, horizon);
    assert_eq!(expected.map(d), result.ok());
}
