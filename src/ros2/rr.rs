use crate::fixed_point;
use crate::supply::SupplyBound;
use crate::time::{Duration, Service};

use crate::arrival::ArrivalBound;
use crate::wcet::JobCostModel;

const EPSILON: Duration = Duration::epsilon();
const EPSILON_SERVICE: Service = Service::in_interval(EPSILON);

/// Relative priority of a polled callback.
///
/// Numerically smaller value == higher priority. This corresponds to
/// the order in which callbacks are registered with the ROS2 runtime
/// system.
pub type PolledCallbackPriority = i32;

/// The priority order among polled callbacks.
///
/// A numerically smaller value corresponds to higher priority.
pub fn is_higher_callback_priority_than(
    a: PolledCallbackPriority,
    b: PolledCallbackPriority,
) -> bool {
    a < b
}

/// Variants of Def. 1 in the paper.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CallbackType {
    /// A timer callback.
    Timer,
    /// An event source pseudo-callback.
    EventSource,
    /// A polled callback, for which we don't know the priority.
    PolledUnknownPrio,
    /// A polled callback, for which we do know the priority.
    Polled(PolledCallbackPriority),
}

impl CallbackType {
    pub fn is_pp(&self) -> bool {
        match self {
            CallbackType::Timer | CallbackType::EventSource => false,
            _ => true,
        }
    }
}

/// A shallow wrapper type for callbacks to compute the direct and
/// self-interference bounds needed for the round-robin-aware
/// analysis (Theorem 2 of the paper).
///
/// Note: this is intentionally not the same type as
/// [bw::Callback][super::bw::Callback], even though they look very
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
    /// **NB**: the `arrival_bound` should conform to the regular
    /// propagation rules as given in Equation (7) in the paper, and
    /// not make any assumptions about busy windows.
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

    /// Direct interference bound (see Def. 1 in the paper).
    fn direct_rbf(
        &self,
        interfered_with: &CallbackType,
        delta: Duration,
        num_polling_points: usize,
    ) -> Service {
        let effective_interval = (delta + self.response_time_bound).saturating_sub(EPSILON);
        let arrived = self.arrival_bound.number_arrivals(effective_interval);
        let n = match self.kind {
            CallbackType::Timer | CallbackType::EventSource => arrived,
            CallbackType::PolledUnknownPrio => arrived.min(num_polling_points + 1),
            CallbackType::Polled(inf_prio) => match *interfered_with {
                CallbackType::Polled(ref_prio) => arrived.min(
                    num_polling_points
                        + is_higher_callback_priority_than(inf_prio, ref_prio) as usize,
                ),
                _ => arrived.min(num_polling_points + 1),
            },
        };
        self.cost_model.cost_of_jobs(n)
    }

    /// A bound on the number of self-interfering instances (see Def. 2 in the paper).
    fn max_self_interfering_instances(&self, delta: Duration) -> usize {
        let effective_interval = (delta + self.response_time_bound).saturating_sub(EPSILON);
        self.arrival_bound
            .number_arrivals(effective_interval)
            .saturating_sub(1)
    }

    /// A bound on self-interference.
    fn self_interference_rbf(&self, delta: Duration) -> Service {
        self.cost_model
            .cost_of_jobs(self.max_self_interfering_instances(delta))
    }

    /// A bound on the marginal execution cost, denoted `\Omega` in Theorem 2.
    fn marginal_execution_cost(&self, delta: Duration) -> Service {
        let n = self.max_self_interfering_instances(delta);
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

/// Round-robin-aware chain analysis (see Theorem 2 in the paper).
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

    // Step 1: find the fixed point S*.

    // compute a bound on the maximum number of polling points in the analysis window
    let max_num_polling_points = Callback::subchain_polling_point_bound(subchain);

    let rhs_S_star = |s_star: Duration| {
        let di = workload
            .iter()
            .map(|cb| {
                if std::ptr::eq(*eoc, cb) {
                    // Don't count the end of the chain, which is
                    // accounted for as self-interference.
                    Service::none()
                } else {
                    cb.direct_rbf(&eoc.kind, s_star, max_num_polling_points)
                }
            })
            .sum();
        let si = eoc.self_interference_rbf(s_star);
        EPSILON_SERVICE + di + si
    };

    let S_star = fixed_point::search(supply, limit, rhs_S_star)?;

    // Step 2: find the response-time bound R*.

    let supply_star = supply.provided_service(S_star);
    let omega = eoc.marginal_execution_cost(S_star);
    let rhs_R_star = supply_star.saturating_sub(EPSILON_SERVICE) + omega;

    // we can directly solve the inequality
    Ok(supply.service_time(rhs_R_star))
}
