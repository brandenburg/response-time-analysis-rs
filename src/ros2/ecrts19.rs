use crate::demand::RequestBound;
use crate::fixed_point::{self, max_response_time, SearchResult};
use crate::supply::SupplyBound;
use crate::time::{Duration, Offset, Service};

/// Bound the response time of an event source for a given supply and
/// demand model, using the analysis proposed by
/// [Casini et al. (2019)](https://people.mpi-sws.org/~bbb/papers/pdf/ecrts19-rev1.pdf).
///
/// If no fixed point is found below the divergence limit given by
/// `limit`, return a [SearchFailure][fixed_point::SearchFailure]
/// instead.
///
/// The bound is based on Lemma 1 of Casini et al. (2019).
pub fn rta_event_source<SBF, RBF>(
    supply: &SBF,
    demand: &RBF,
    limit: Duration,
) -> fixed_point::SearchResult
where
    SBF: SupplyBound + ?Sized,
    RBF: RequestBound + ?Sized,
{
    // right-hand side of Lemma 6 --- used to bound the max. busy-window length
    let rhs_busy_window = |delta| demand.service_needed(delta);
    // right-hand side of Lemma 1 --- used to bound the demand for a given offset
    let rhs = |offset: Offset, _response| demand.service_needed(offset.closed_since_time_zero());

    // solve the fixed point for all steps of the demand curve up to
    // the maximum busy-window length and return the maximum (or divergence)
    bound_response_time(supply, demand, rhs_busy_window, rhs, limit)
}

/// Bound the response time of a timer callback using
/// Lemma 3 of
/// [Casini et al. (2019)](https://people.mpi-sws.org/~bbb/papers/pdf/ecrts19-rev1.pdf).
///
/// # Parameters
///
/// - `supply`: the supply model of the shared executor
/// - `own_demand`: model of the processor demand of the timer under analysis
/// - `interfering_demand`: model of the processor demand of higher-priority timers
/// - `blocking_bound`: a bound on the maximum delay due to lower-priority callbacks
/// - `limit`: the divergence threshold at which the search for a fixed point is aborted
///
/// If no fixed point is found below the divergence limit given by
/// `limit`, return a [SearchFailure][fixed_point::SearchFailure]
/// instead.
pub fn rta_timer<SBF, RBF1, RBF2>(
    supply: &SBF,
    own_demand: &RBF1,
    interfering_demand: &RBF2,
    blocking_bound: Service,
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
    let rhs = |offset: Offset, response: Duration| {
        // length of the prefix interval before the offset
        let prefix = offset.since_time_zero();
        // cost of timer callback
        let own_wcet = Duration::from(own_demand.least_wcet_in_interval(prefix + response));
        // determine timeframe during which other callbacks can delay us
        let interference_interval = if response > own_wcet {
            prefix + response - own_wcet + Duration::epsilon()
        } else {
            prefix + Duration::epsilon()
        };
        own_demand.service_needed(prefix + Duration::epsilon())
            + interfering_demand.service_needed(interference_interval)
            + blocking_bound
    };
    bound_response_time(supply, own_demand, rhs_bw, rhs, limit)
}

/// Bound the response time of a polling-point-based callback using
/// Lemmas 4 and 5 of
/// [Casini et al. (2019)](https://people.mpi-sws.org/~bbb/papers/pdf/ecrts19-rev1.pdf).
///
/// # Parameters
///
/// - `supply`: the supply model of the shared executor
/// - `own_demand`: model of the processor demand of the callback under analysis
/// - `interfering_demand`: model of the processor demand of all higher-priority callbacks
/// - `limit`: the divergence threshold at which the search for a fixed point is aborted
///
/// If no fixed point is found below the divergence limit given by
/// `limit`, return a [SearchFailure][fixed_point::SearchFailure]
/// instead.
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
    // right-hand side of Eq (6), based on Lemmas 4 and 5
    let rhs = |offset: Offset, response: Duration| {
        // length of the prefix interval before the offset
        let prefix = offset.since_time_zero();
        // cost of pp-based callback under analysis
        let own_wcet = Duration::from(own_demand.least_wcet_in_interval(prefix + response));
        // determine timeframe during which other callbacks can delay us
        let interference_interval = if response > own_wcet {
            prefix + response - own_wcet + Duration::epsilon()
        } else {
            prefix + Duration::epsilon()
        };
        own_demand.service_needed(prefix + Duration::epsilon())
            + interfering_demand.service_needed(interference_interval)
    };
    bound_response_time(supply, own_demand, rhs_bw, rhs, limit)
}

/// Bound the response time of a chain of callbacks using
/// Lemma 8 of
/// [Casini et al. (2019)](https://people.mpi-sws.org/~bbb/papers/pdf/ecrts19-rev1.pdf).
///
/// # Parameters
///
/// - `supply`: the supply model of the shared executor
/// - `chain_last_callback`: model of the processor demand of the last
///   callback of the chain under analysis
/// - `chain_prefix`: model of the processor demand of the callbacks
///   in the chain under analysis preceding the last callback
/// - `full_chain`: model of the processor demand of the chain under analysis --- obviously, this must be consistent with the two prior parameters
/// - `other_chains`: model of the processor demand of other callback
///   chains allocated to the same executor
/// - `limit`: the divergence threshold at which the search for a fixed point is aborted
///
/// If no fixed point is found below the divergence limit given by
/// `limit`, return a [SearchFailure][fixed_point::SearchFailure]
/// instead.
pub fn rta_processing_chain<SBF, RBF1, RBF2, RBF3, RBF4>(
    supply: &SBF,
    chain_last_callback: &RBF1,
    chain_prefix: &RBF2,
    full_chain: &RBF3,
    other_chains: &RBF4,
    limit: Duration,
) -> fixed_point::SearchResult
where
    SBF: SupplyBound + ?Sized,
    RBF1: RequestBound + ?Sized,
    RBF2: RequestBound + ?Sized,
    RBF3: RequestBound + ?Sized,
    RBF4: RequestBound + ?Sized,
{
    // for bounding the max. busy-window length
    let rhs_bw = |delta| {
        debug_assert_eq!(
            full_chain.service_needed(delta),
            chain_prefix.service_needed(delta) + chain_last_callback.service_needed(delta)
        );
        full_chain.service_needed(delta) + other_chains.service_needed(delta)
    };

    // right-hand side of Lemma 8
    let rhs = |offset: Offset, response: Duration| {
        let prefix = offset.since_time_zero();
        let own_demand = chain_last_callback.service_needed(prefix + Duration::epsilon());
        let own_wcet =
            Duration::from(chain_last_callback.least_wcet_in_interval(prefix + response));
        // determine timeframe during which other callbacks can delay us
        let interference_interval = if response > own_wcet {
            prefix + response - own_wcet + Duration::epsilon()
        } else {
            prefix + Duration::epsilon()
        };
        let other_demand = other_chains.service_needed(interference_interval);
        let self_interference = chain_prefix.service_needed(interference_interval);
        own_demand + self_interference + other_demand
    };

    bound_response_time(supply, full_chain, rhs_bw, rhs, limit)
}

/// Try to find a response-time bound for a given processor supply
/// model and a given processor demand model.
///
/// The search for a fixed point will be aborted if the given
/// divergence threshold indicated by `limit` is reached.
///
/// The fixed-point search relies on three relevant characterizations
/// of processor demand:
/// - `demand` is demand model of the callback under analysis from
///    which all points are inferred at which the demand curve
///    exhibits "steps".
/// - `bw_demand_bound` is the right-hand side of the fixed-point
///   equation describing the maximum busy-window length, i.e., the
///   demand of "everything".
/// - `offset_demand_bound` is the right-hand side of the fixed-point
///   equation describing the response time for a given offset.
fn bound_response_time<SBF, RBF, F, G>(
    supply: &SBF,
    demand: &RBF,
    bw_demand_bound: F,
    offset_demand_bound: G,
    limit: Duration,
) -> SearchResult
where
    SBF: SupplyBound + ?Sized,
    RBF: RequestBound + ?Sized,
    F: Fn(Duration) -> Service,
    G: Fn(Offset, Duration) -> Service,
{
    // find a bound on the maximum busy-window
    let max_bw = fixed_point::search(supply, limit, bw_demand_bound)?;
    // Consider the search space of relevant offsets based on Lemma 7.
    // That is, we have to look at all points where the demand curve "steps".
    let offsets = demand
        .steps_iter()
        // Note that steps_iter() yields interval lengths, but we are interested in
        // offsets. Since the length of an interval [0, A] is A+1, we need to subtract one
        // to obtain the offset.
        .map(Offset::closed_from_time_zero)
        .take_while(|x| *x <= Offset::from_time_zero(max_bw));
    // for each relevant offset in the search space,
    let rta_bounds = offsets.map(|offset| {
        let rhs = |delta| offset_demand_bound(offset, delta);
        fixed_point::search_with_offset(supply, offset, limit, &rhs)
    });
    max_response_time(rta_bounds)
}
