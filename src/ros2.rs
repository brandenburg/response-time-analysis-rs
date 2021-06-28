use crate::arrivals::ArrivalBound;
use crate::demand::{self, AggregateRequestBound, RequestBound};
use crate::fixed_point;
use crate::supply::SupplyBound;
use crate::time::Duration;
use crate::wcet::Scalar as ScalarWCET;

pub fn rta_event_source<SBF, RBF>(
    supply: &SBF,
    demand: &RBF,
    limit: Duration,
) -> fixed_point::SearchResult
where
    SBF: SupplyBound + ?Sized,
    RBF: RequestBound + ?Sized,
{
    // right-hand side of Lemma 6
    let rhs_busy_window = |delta| demand.service_needed(delta);
    // right-hand side of Lemma 1
    let rhs = |offset, _response| demand.service_needed(offset + 1);
    fixed_point::bound_response_time(supply, demand, rhs_busy_window, rhs, limit)
}

pub fn rta_timer<SBF, RBF1, RBF2>(
    supply: &SBF,
    own_demand: &RBF1,
    interfering_demand: &RBF2,
    blocking_bound: Duration,
    limit: Duration,
) -> fixed_point::SearchResult
where
    SBF: SupplyBound + ?Sized,
    RBF1: RequestBound + ?Sized,
    RBF2: RequestBound + ?Sized,
{
    // right-hand side of Lemma 6
    let rhs_bw = |delta| {
        own_demand.service_needed(delta) + blocking_bound + interfering_demand.service_needed(delta)
    };
    // right-hand side of Lemma 3
    let rhs = |offset, response| {
        // cost of timer callback
        let own_wcet = own_demand.least_wcet_in_interval(offset + response);
        // determine timeframe during which other callbacks can delay us
        let interference_interval = if response > own_wcet {
            offset + response - own_wcet + 1
        } else {
            offset + 1
        };
        own_demand.service_needed(offset + 1)
            + interfering_demand.service_needed(interference_interval)
            + blocking_bound
    };
    fixed_point::bound_response_time(supply, own_demand, rhs_bw, rhs, limit)
}

pub fn rta_polling_point_callback<SBF, RBF1, RBF2>(
    supply: &SBF,
    own_demand: &RBF1,
    interfering_demand: &RBF2,
    limit: Duration,
) -> fixed_point::SearchResult
where
    SBF: SupplyBound + ?Sized,
    RBF1: RequestBound + ?Sized,
    RBF2: RequestBound + ?Sized,
{
    // right-hand side of Lemma 6
    let rhs_bw =
        |delta| own_demand.service_needed(delta) + interfering_demand.service_needed(delta);
    // right-hand side of Lemma 3
    let rhs = |offset, response| {
        // cost of pp-based callback under analysis
        let own_wcet = own_demand.least_wcet_in_interval(offset + response);
        // determine timeframe during which other callbacks can delay us
        let interference_interval = if response > own_wcet {
            offset + response - own_wcet + 1
        } else {
            offset + 1
        };
        own_demand.service_needed(offset + 1)
            + interfering_demand.service_needed(interference_interval)
    };
    fixed_point::bound_response_time(supply, own_demand, rhs_bw, rhs, limit)
}

pub fn rta_processing_chain<SBF, RBF>(
    supply: &SBF,
    all_chains: &RBF,
    last_callback_wcet: Duration,
    limit: Duration,
) -> fixed_point::SearchResult
where
    SBF: SupplyBound + ?Sized,
    RBF: RequestBound + ?Sized,
{
    // right-hand side of Lemma 8
    let rhs = |response| {
        if response > last_callback_wcet {
            all_chains.service_needed(response - last_callback_wcet + 1)
        } else {
            all_chains.service_needed(1)
        }
    };
    fixed_point::search(supply, limit, rhs)
}

// NOTE: Just a sketch, no proof of correctness yet.
pub fn rta_processing_chain_window_aware<SBF, AB, RBF>(
    supply: &SBF,
    chain_costs: impl Iterator<Item = Duration>,
    chain_arrival_bound: &AB,
    interfering_demand: &RBF,
    limit: Duration,
) -> fixed_point::SearchResult
where
    SBF: SupplyBound + ?Sized,
    AB: ArrivalBound + Clone,
    RBF: AggregateRequestBound + ?Sized,
{
    // split costs by prefix and last callback in chain, and count callbacks
    let (prefix_cost, last_cb, chain_length) = {
        // let's define these as mutable only while we determine them
        let mut prefix_cost = 0;
        let mut last_cb = 0;
        let mut chain_length = 0;

        for c in chain_costs {
            prefix_cost += c;
            last_cb = c;
            chain_length += 1;
        }
        prefix_cost -= last_cb;
        // initialize with immutable final values
        (prefix_cost, last_cb, chain_length)
    };

    let prefix_rbf = demand::RBF {
        wcet: ScalarWCET::from(prefix_cost),
        arrival_bound: chain_arrival_bound.clone(),
    };

    let suffix_rbf = demand::RBF {
        wcet: ScalarWCET::from(last_cb),
        arrival_bound: chain_arrival_bound.clone(),
    };

    // busy-window ends when all chains are quiet
    let rhs_bw = |delta| {
        prefix_rbf.service_needed(delta)
            + suffix_rbf.service_needed(delta)
            + interfering_demand.service_needed(delta)
    };
    // right-hand side of recurrence for chain analysis
    let rhs = |offset, response| {
        // compute the maximum number of relevant processing windows
        let num_windows = chain_arrival_bound.number_arrivals(offset + 1) * chain_length;

        let interference_interval = if response > last_cb {
            offset + response - last_cb + 1
        } else {
            offset + 1
        };
        suffix_rbf.service_needed(offset + 1)
            + prefix_rbf.service_needed_by_n_jobs(interference_interval, num_windows)
            + interfering_demand
                .service_needed_by_n_jobs_per_component(interference_interval, 1 + num_windows)
    };
    fixed_point::bound_response_time(supply, &suffix_rbf, rhs_bw, rhs, limit)
}
