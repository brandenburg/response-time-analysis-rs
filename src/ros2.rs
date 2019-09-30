use crate::analysis::{self, RequestBound};
use crate::supply::SupplyBound;
use crate::time::*;

pub fn rta_event_source<SBF, RBF>(supply: &SBF, demand: &RBF, limit: Duration) -> Option<Duration>
where
    SBF: SupplyBound,
    RBF: RequestBound,
{
    // right-hand side of Lemma 6
    let rhs_busy_window = |delta| demand.service_needed(delta);
    // right-hand side of Lemma 1
    let rhs = |offset, _response| demand.service_needed(offset + 1);
    analysis::bound_response_time(supply, demand, rhs_busy_window, rhs, limit)
}

pub fn rta_timer<SBF, RBF1, RBF2>(
    supply: &SBF,
    own_demand: &RBF1,
    interfering_demand: &RBF2,
    blocking_bound: Duration,
    limit: Duration,
) -> Option<Duration>
where
    SBF: SupplyBound,
    RBF1: RequestBound,
    RBF2: RequestBound,
{
    // cost of timer callback
    let own_wcet = own_demand.max_single_job_cost();
    // right-hand side of Lemma 6
    let rhs_bw = |delta| {
        own_demand.service_needed(delta) + blocking_bound + interfering_demand.service_needed(delta)
    };
    // right-hand side of Lemma 3
    let rhs = |offset, response| {
        own_demand.service_needed(offset + 1) +
        interfering_demand.service_needed(offset + response - own_wcet + 1) +
        blocking_bound
    };
    analysis::bound_response_time(supply, own_demand, rhs_bw, rhs, limit)
}

pub fn rta_polling_point_callback<SBF, RBF1, RBF2>(
    supply: &SBF,
    own_demand: &RBF1,
    interfering_demand: &RBF2,
    limit: Duration,
) -> Option<Duration>
where
    SBF: SupplyBound,
    RBF1: RequestBound,
    RBF2: RequestBound,
{
    // cost of pp-based callback under analysis
    let own_wcet = own_demand.max_single_job_cost();
    // right-hand side of Lemma 6
    let rhs_bw = |delta| {
        own_demand.service_needed(delta) + interfering_demand.service_needed(delta)
    };
    // right-hand side of Lemma 3
    let rhs = |offset, response| {
        own_demand.service_needed(offset + 1) +
        interfering_demand.service_needed(offset + response - own_wcet + 1)
    };
    analysis::bound_response_time(supply, own_demand, rhs_bw, rhs, limit)
}

pub fn rta_processing_chain<SBF, RBF>(
    supply: &SBF,
    all_chains: &RBF,
    last_callback_wcet: Duration,
    limit: Duration,
) -> Option<Duration>
where
    SBF: SupplyBound,
    RBF: RequestBound,
{
    // right-hand side of Lemma 3
    let rhs = |response| {
        if response > last_callback_wcet {
            all_chains.service_needed(response - last_callback_wcet + 1)
        } else {
            all_chains.service_needed(1)
        }
    };
    analysis::fixed_point_search(supply, limit, rhs)
}
