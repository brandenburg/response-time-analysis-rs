use itertools::Itertools;

use crate::fixed_point;
use crate::supply::SupplyBound;
use crate::time::{Duration, Offset, Service};

use crate::arrival::ArrivalBound;
use crate::wcet::JobCostModel;

pub use super::rr::{is_higher_callback_priority_than, CallbackType};

const EPSILON: Duration = Duration::epsilon();
const EPSILON_SERVICE: Service = Service::in_interval(EPSILON);

/// A shallow wrapper type for callbacks to compute the direct and
/// self-interference bounds needed for the busy-window- and
/// round-robin-aware analysis (Theorem 3 of [the paper](https://people.mpi-sws.org/~bbb/papers/pdf/rtss21-ros.pdf)).
///
/// Note: this is intentionally not the same type as
/// [rr::Callback][super::rr::Callback], even though they look very
/// similar, since different assumptions on the wrapped
/// `arrival_bound` are made.
pub struct Callback<'a, 'b, AB: ArrivalBound + ?Sized, CM: JobCostModel + ?Sized> {
    response_time_bound: Duration,
    arrival_bound: &'a AB,
    cost_model: &'b CM,
    kind: CallbackType,
}

impl<'a, 'b, AB: ArrivalBound + ?Sized, CM: JobCostModel + ?Sized> Callback<'a, 'b, AB, CM> {
    /// Create a new wrapper around a given arrival model, cost
    /// model, and assumed response-time bound.
    ///
    /// **NB**: the `arrival_bound` should conform to the
    /// busy-window-aware propagation rules (Definition 4 in the
    /// paper).
    pub fn new(
        response_time_bound: Duration,
        arrival_bound: &'a AB,
        cost_model: &'b CM,
        kind: CallbackType,
    ) -> Callback<'a, 'b, AB, CM> {
        Callback {
            response_time_bound,
            arrival_bound,
            cost_model,
            kind,
        }
    }

    /// Busy-window interference bound (see Def. 5 in the paper).
    fn busy_window_rbf(
        &self,
        interfered_with: &CallbackType,
        delta: Duration,
        activation_time: Offset,
        num_polling_points: usize,
    ) -> Service {
        // arrivals in the half-open interval [0, delta)
        let arrived = self.arrival_bound.number_arrivals(delta);
        // arrivals in the half-open interval [0, activation_time)
        let arrived_bw = self
            .arrival_bound
            .number_arrivals(activation_time.since_time_zero())
            + num_polling_points;
        let n = match self.kind {
            CallbackType::Timer | CallbackType::EventSource => arrived,
            CallbackType::PolledUnknownPrio => arrived.min(arrived_bw + 1),
            CallbackType::Polled(inf_prio) => match *interfered_with {
                CallbackType::Polled(ref_prio) => arrived.min(
                    arrived_bw + is_higher_callback_priority_than(inf_prio, ref_prio) as usize,
                ),
                _ => arrived.min(arrived_bw + 1),
            },
        };
        self.cost_model.cost_of_jobs(n)
    }

    /// A bound on the maximum number of self-interfering instances
    /// in a busy window, where the instance under analysis is
    /// activated at the given activation offset.
    fn max_self_interfering_instances(&self, activation: Offset) -> usize {
        // activations in the closed interval [0, activation]
        self.arrival_bound
            .number_arrivals(activation.closed_since_time_zero())
            .saturating_sub(1)
    }

    /// A bound on self-interference in a busy window, where the
    /// instance under analysis is activated at the given activation
    /// offset.
    fn self_interference_rbf(&self, activation: Offset) -> Service {
        self.cost_model
            .cost_of_jobs(self.max_self_interfering_instances(activation))
    }

    /// A bound on the total workload of the callback in a busy
    /// window, where `A_star` denotes the end point of the half-open
    /// busy-window interval `[0, A_star)`.
    ///
    /// See Lemma 18 of [the paper](https://people.mpi-sws.org/~bbb/papers/pdf/rtss21-ros.pdf).
    #[allow(non_snake_case)]
    fn own_workload_rbf(&self, A_star: Offset) -> Service {
        let n_activations = self.arrival_bound.number_arrivals(A_star.since_time_zero());
        self.cost_model.cost_of_jobs(n_activations)
    }

    /// A bound on the marginal execution cost, denoted `\Omega` in Theorem 2.
    fn marginal_execution_cost(&self, activation: Offset) -> Service {
        let n = self.max_self_interfering_instances(activation);
        self.cost_model.cost_of_jobs(n + 1) - self.cost_model.cost_of_jobs(n)
    }

    /// A bound on the maximum number of polling points "incurred"
    /// (see Def. 3 in the paper).
    fn polling_point_bound(&self) -> usize {
        self.arrival_bound.number_arrivals(self.response_time_bound)
    }

    /// The total polling-point bound of a subchain (i.e., a sequence of
    /// callbacks). The subchain is expressed as a slice of callback
    /// references. See Def. 3 in the paper.
    fn subchain_polling_point_bound(subchain: &[&Callback<AB, CM>]) -> usize {
        subchain.iter().map(|cb| cb.polling_point_bound()).sum()
    }
}

/// Hybrid round-robin- and busy-window-aware chain analysis (Theorem
/// 3 in [the paper](https://people.mpi-sws.org/~bbb/papers/pdf/rtss21-ros.pdf)).
///
/// # Parameters
///
/// - `supply`: the supply model of the shared executor
/// - `workload`: all callbacks served by the shared executor
/// - `subchain`: the subchain under analysis --- each reference must
///               be unique and point to one of the callbacks in `workload`
/// - `limit`: the divergence threshold at which the search for a
///   fixed point is aborted
///
/// If no fixed point is found below the divergence limit given by
/// `limit`, return a [SearchFailure][fixed_point::SearchFailure]
/// instead.
#[allow(non_snake_case)]
pub fn rta_subchain<SBF, AB, CM>(
    supply: &SBF,
    workload: &[Callback<AB, CM>],
    subchain: &[&Callback<AB, CM>],
    limit: Duration,
) -> fixed_point::SearchResult
where
    SBF: SupplyBound + ?Sized,
    AB: ArrivalBound + ?Sized,
    CM: JobCostModel + ?Sized,
{
    // First, compute a bound on the maximum number of polling points
    // in the analysis window.
    let max_num_polling_points = Callback::subchain_polling_point_bound(subchain);

    // callback at the end of the chain under analysis
    let eoc = subchain.last().expect("subchain must not be empty");

    // check that we are actually given a proper subchain, i.e., all references
    // in the subchain must point to something in the workload
    debug_assert!(
        subchain
            .iter()
            .all(|sc_cb| workload.iter().any(|wl_cb| std::ptr::eq(*sc_cb, wl_cb))),
        "subchain not wholly part of workload"
    );

    // Busy-window-aware interference (Def. 5)
    let bw_interference = |delta: Duration, activation: Offset| {
        workload
            .iter()
            .map(|cb| {
                if std::ptr::eq(*eoc, cb) {
                    // Don't count the end of the chain, which is
                    // accounted for as self-interference.
                    Service::none()
                } else {
                    cb.busy_window_rbf(&eoc.kind, delta, activation, max_num_polling_points)
                }
            })
            .sum()
    };

    // The response-time analysis for a given offset (Theorem 3).
    let rta = |activation: Offset| -> fixed_point::SearchResult {
        // Step 1: bound the starting time S*.
        let si = eoc.self_interference_rbf(activation);

        let rhs_S_star = |S_star: Duration| {
            let di = bw_interference(S_star, activation);
            EPSILON_SERVICE + di + si
        };

        let S_star = fixed_point::search(supply, limit, rhs_S_star)?;

        // Step 2: find the response-time bound R*.

        let supply_star = supply.provided_service(S_star);
        let omega = eoc.marginal_execution_cost(activation);
        let rhs_F_star = supply_star.saturating_sub(EPSILON_SERVICE) + omega;
        let F_star = supply.service_time(rhs_F_star);

        // the response-time bound
        Ok(F_star.saturating_sub(activation.since_time_zero()))
    };

    // Bound the maximum offset (Lemma 18).
    let rhs_max = |A_star: Duration| {
        let si = eoc.own_workload_rbf(Offset::from_time_zero(A_star));
        let di = bw_interference(A_star, Offset::from_time_zero(A_star));
        EPSILON_SERVICE + di + si
    };
    let max_offset = Offset::from_time_zero(fixed_point::search(supply, limit, rhs_max)?);

    // Define the search space of relevant offsets (Lemma 19).
    // First, find all relevant steps.
    let all_steps = workload
        .iter()
        // we only care about polled callbacks and the end of the
        // subchain under analysis
        .filter(|cb| cb.kind.is_pp() || std::ptr::eq(*eoc, *cb))
        .map(|cb| {
            cb.arrival_bound.steps_iter().map(move |delta| {
                if std::ptr::eq(*eoc, cb) {
                    // This is the callback under analysis.
                    // The steps_iter() gives us the values of delta such that
                    // number_arrivals(delta-1) < number_arrivals_delta().
                    // However, we are looking for A such that
                    // number_arrivals(A) < number_arrivals(A + 1).
                    // Thus, we need to subtract one from delta.
                    Offset::from_time_zero(delta.saturating_sub(EPSILON))
                } else {
                    // This is some interfering polled callback.
                    // The steps_iter() gives us the values of delta such that
                    // number_arrivals(delta-1) < number_arrivals_delta().
                    // That's exactly what we are looking for here, so we
                    // can just pass it through.
                    Offset::from_time_zero(delta)
                }
            })
        })
        .kmerge()
        .dedup();

    // In a debug build, let's double-check the steps computed above
    // with a brute-force solution.
    #[cfg(debug_assertions)]
    let mut brute_force_steps = (0..)
        .filter(|A| {
            workload.iter().any(|cb|
                // Negated conditions of Lemma 19.
                if std::ptr::eq(*eoc, cb) {
                    cb.arrival_bound.number_arrivals(Duration::from(*A)) !=
                    cb.arrival_bound.number_arrivals(Duration::from(*A + 1))
                } else {
                    cb.kind.is_pp() && *A > 0 &&
                    cb.arrival_bound.number_arrivals(Duration::from(*A - 1)) !=
                    cb.arrival_bound.number_arrivals(Duration::from(*A))
                }
            )
        })
        .map(Offset::from)
        .peekable();

    // In a debug build, shadow all_steps with the checked version.
    #[cfg(debug_assertions)]
    let all_steps = {
        let mut wrapped = all_steps.peekable();
        // Manually check the first point to make sure we're not calling
        // zip on an empty iterator.
        assert_eq!(brute_force_steps.peek(), wrapped.peek());
        wrapped.zip(brute_force_steps).map(|(a, bf)| {
            assert_eq!(a, bf);
            a
        })
    };

    // The search space is given by A=0 and each relevant step below max_offset.
    let search_space = all_steps.take_while(|activation| *activation < max_offset);

    // Apply the offset-specific RTA to each offset in the search space and
    // return the maximum response-time bound.
    fixed_point::max_response_time(search_space.map(rta))
}
