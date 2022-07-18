//! RTA for FP scheduling with limited-preemptive jobs (**LP-FP**)

use crate::arrival::ArrivalBound;
use crate::demand::{self, RequestBound};
use crate::time::{Duration, Offset, Service};
use crate::{fixed_point, supply, wcet};

/// The information about the task under analysis required to perform
/// the analysis.
/// Create one struct of this type to represent the task under analysis.
pub struct TaskUnderAnalysis<'a, AB: ArrivalBound + ?Sized> {
    /// The task's WCET.
    pub wcet: wcet::Scalar,

    /// The task's arrival bound.
    pub arrivals: &'a AB,

    /// The maximum length of the task's last segment.
    pub last_np_segment: Service,

    /// The `blocking_bound` must be a bound on the maximum priority
    /// inversion caused by tasks of lower priority, which corresponds
    /// to the maximum segment length of any lower-priority task.
    pub blocking_bound: Service,
}

/// Try to find a response-time bound for a task under
/// limited-preemptive fixed-priority scheduling on a dedicated
/// uniprocessor.
///
/// The analysis assumes that all tasks are independent and that each
/// is characterized by an arbitrary arrival curve and a WCET bound.
/// The set of higher-or-equal-priority tasks is represented by
/// `interfering_tasks`; the task under analysis is given by
/// `tua`.
///
/// If no fixed point is found below the divergence limit given by
/// `limit`, the function returns a
/// [SearchFailure][fixed_point::SearchFailure] instead.
///
/// This analysis is an implementation of the corresponding  verified
/// instantiation of [the abstract RTA of Bozhko and Brandenburg
/// (ECRTS 2020)](https://drops.dagstuhl.de/opus/volltexte/2020/12385/pdf/LIPIcs-ECRTS-2020-22.pdf).
/// See also [the Coq-verified instantiation](https://prosa.mpi-sws.org/branches/master/pretty/prosa.results.fixed_priority.rta.limited_preemptive.html).
#[allow(non_snake_case)]
pub fn dedicated_uniproc_rta<InterferingRBF, AB>(
    tua: &TaskUnderAnalysis<AB>,
    interfering_tasks: &[InterferingRBF],
    limit: Duration,
) -> fixed_point::SearchResult
where
    InterferingRBF: RequestBound,
    AB: ArrivalBound + ?Sized,
{
    // This analysis is specific to dedicated uniprocessors.
    let proc = supply::Dedicated::new();

    // For convenience, define the RBF for the task under analysis.
    let tua_rbf = demand::RBF::new(tua.arrivals, tua.wcet);

    // First, bound the maximum possible busy-window length.
    let L = fixed_point::search(&proc, limit, |L| {
        let interference_bound: Service = interfering_tasks
            .iter()
            .map(|rbf| rbf.service_needed(L))
            .sum();

        tua.blocking_bound + interference_bound + tua_rbf.service_needed(L)
    })?;

    // Second, the run-to-completion threshold of the task under
    // analysis. In the limited preemptive case, no job can be preempted
    // after it reaches its last non-preemptive segment.
    // See also: https://prosa.mpi-sws.org/branches/master/pretty/prosa.model.task.preemption.limited_preemptive.html#limited_preemptive
    let rtct = tua.wcet.wcet - (tua.last_np_segment - Service::epsilon());
    // The remaining cost after the run-to-completion threshold has been reached.
    let rem_cost = tua.wcet.wcet - rtct;

    // Now define the offset-specific RTA.
    let rta = |A: Offset| {
        // Define the RHS of the equation in theorem 31 of the aRTA paper,
        // where AF = A + F.
        let rhs = |AF: Duration| {
            // demand of the task under analysis
            let self_interference = tua_rbf.service_needed(A.closed_since_time_zero());
            let tua_demand = self_interference - rem_cost;

            // demand of all interfering tasks
            let interfering_demand = interfering_tasks
                .iter()
                .map(|rbf| rbf.service_needed(AF))
                .sum();

            // considering `blocking_bound` to account for priority inversion
            tua.blocking_bound + tua_demand + interfering_demand
        };

        // Find the solution A+F that is the least fixed point
        let AF = fixed_point::search(&proc, limit, rhs)?;
        // Extract the corresponding bound.
        let F = AF - A.since_time_zero();
        Ok(F + Duration::from(rem_cost))
    };

    // Third, define the search space. The search space is given by
    // A=0 and each step below L of the task under analysis's RBF.
    // The case of A=0 is not handled explicitly since `step_offsets()`
    // necessarily yields it.
    let max_offset = Offset::from_time_zero(L);
    let search_space = demand::step_offsets(&tua_rbf).take_while(|A| *A < max_offset);

    // Apply the offset-specific RTA to each offset in the search space and
    // return the maximum response-time bound.
    fixed_point::max_response_time(search_space.map(rta))
}
