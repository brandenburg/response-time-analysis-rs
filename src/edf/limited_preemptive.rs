//! RTA for EDF scheduling with limited-preemptive jobs (**LP-EDF**)

use itertools::Itertools;

use crate::arrival::ArrivalBound;
use crate::demand::{self, RequestBound};
use crate::time::{Duration, Offset, Service};
use crate::{fixed_point, supply, wcet};

/// The information about the task under analysis required to perform
/// the analysis.
/// Create one struct of this type to represent the task under analysis.
pub struct TaskUnderAnalysis<'a, AB: ArrivalBound + ?Sized> {
    /// The task's worst-case execution time.
    pub wcet: wcet::Scalar,
    /// The task's arrival curve.
    pub arrivals: &'a AB,
    /// The task's relative deadline.
    pub deadline: Duration,
    /// An upper bound on the length of the task's last segment.
    pub last_np_segment: Service,
}

/// The per-task information required to account for interfering tasks.
/// Create a struct of this type for each interfering task.
pub struct InterferingTask<'a, RBF: RequestBound + ?Sized> {
    /// The RBF upper-bounding the task's demand.
    pub rbf: &'a RBF,
    /// The task's relative deadline.
    pub deadline: Duration,
    /// An upper bound on the task's maximum segment length.
    pub max_np_segment: Service,
}

/// Bound the maximum response time of a task under *limited-preemptive
/// earliest-deadline first* (**LP-EDF**) scheduling on a dedicated,
/// ideal uniprocessor.
///
/// That is, the analysis assumes that:
///
/// 1. The processor is available to the tasks 100% of the time.
/// 2. Scheduling overheads are negligible (i.e., already integrated
///    into the task parameters).
/// 3. Jobs are prioritized in order of increasing absolute deadlines
///    (with ties broken arbitrarily).
/// 4. Each job executes a sequence of nonpreemptive segments.
/// 5. Jobs cannot be preempted within segments.
/// 6. Between any two segments there is a preemption point at which a
///    job may be preempted.
/// 7. The maximum segment length is known for all interfering tasks.
/// 8. The length of the last segment is known for the task under analysis.
///
/// The analysis further assumes that all tasks are independent and that each
/// task is characterized by an arbitrary arrival curve and a WCET
/// bound.
///
/// This analysis is an implementation of the corresponding  verified
/// instantiation of [the abstract RTA of Bozhko and Brandenburg
/// (ECRTS 2020)](https://drops.dagstuhl.de/opus/volltexte/2020/12385/pdf/LIPIcs-ECRTS-2020-22.pdf).
/// Refer to the [the Coq-verified instantiation](http://prosa.mpi-sws.org/branches/master/pretty/prosa.results.edf.rta.limited_preemptive.html)
/// for the latest version.
///
/// The task for which a response-time bound is to be found is
/// represented by `tua`, the set of interfering tasks that share the
/// same processor is given by `other_tasks`.
///
/// If no fixed point is found below the divergence limit given by
/// `limit`, the function returns a
/// [SearchFailure][fixed_point::SearchFailure] instead.
#[allow(non_snake_case)]
pub fn dedicated_uniproc_rta<RBF, AB>(
    tua: &TaskUnderAnalysis<AB>,
    other_tasks: &[InterferingTask<RBF>],
    limit: Duration,
) -> fixed_point::SearchResult
where
    RBF: RequestBound + ?Sized,
    AB: ArrivalBound + ?Sized,
{
    // This analysis is specific to dedicated uniprocessors.
    let proc = supply::Dedicated::new();

    // For convenience, define the RBF for the task under analysis.
    let tua_rbf = demand::RBF::new(tua.arrivals, &tua.wcet);

    // First, bound the maximum possible busy-window length.
    let L = fixed_point::search(&proc, limit, |L| {
        let interference_bound: Service =
            other_tasks.iter().map(|ot| ot.rbf.service_needed(L)).sum();
        interference_bound + tua_rbf.service_needed(L)
    })?;

    // Second, define the RTA for a given offset A. To this end, we
    // first define some components of the fixed-point equation.

    // The run-to-completion threshold of the task under analysis. Given
    // limited preemptive case no job can be preempted after a job
    // reaches its last non-preemptive segment.
    // See also: https://prosa.mpi-sws.org/branches/master/pretty/prosa.model.task.preemption.limited_preemptive.html#limited_preemptive
    let rtct = tua.wcet.wcet - (tua.last_np_segment - Service::epsilon());

    // The remaining cost after the run-to-completion threshold has been
    // reached.
    let rem_cost = tua.wcet.wcet - rtct;

    // Now define the offset-specific RTA.
    let rta = |A: Offset| {
        // Bound on the priority inversion caused by jobs with lower priority.
        let blocking_bound = other_tasks
            .iter()
            .filter(|ot| {
                ot.deadline > tua.deadline + A.since_time_zero()
                    && ot.rbf.service_needed(Duration::epsilon()) > Service::none()
            })
            .map(|ot| ot.max_np_segment.saturating_sub(Service::epsilon()))
            .max()
            .unwrap_or_else(Service::none);

        // Define the RHS of the equation in theorem 31 of the aRTA paper,
        // where AF = A + F.
        let rhs = |AF: Duration| {
            // demand of the task under analysis
            let self_interference = tua_rbf.service_needed(A.closed_since_time_zero());

            let tua_demand = self_interference - rem_cost;

            // demand of all interfering tasks
            let bound_on_total_hep_workload: Service = other_tasks
                .iter()
                .map(|ot| {
                    ot.rbf.service_needed(std::cmp::min(
                        AF,
                        (Offset::since_time_zero(A) + Duration::epsilon() + tua.deadline)
                            .saturating_sub(ot.deadline),
                    ))
                })
                .sum();

            // considering `blocking_bound` to account for priority inversion.
            blocking_bound + tua_demand + bound_on_total_hep_workload
        };

        // Find the solution A+F that is the least fixed point.
        let AF = fixed_point::search(&proc, limit, rhs)?;
        // Extract the corresponding bound.
        let F = AF.saturating_sub(A.since_time_zero());

        Ok(F + Duration::from(rem_cost))
    };

    // Third, define the search space. The search space is given by
    // A=0 and each step below L of the task under analysis's RBF.
    // The case of A=0 is not handled explicitly since `steps_iter()`
    // necessarily yields delta=1, which results in A=0 being
    // included in the search space.
    let max_offset = Offset::from_time_zero(L);
    let search_space_tua = demand::step_offsets(&tua_rbf).take_while(|A| *A < max_offset);
    let search_space = other_tasks
        .iter()
        .map(|ot| {
            demand::step_offsets(ot.rbf)
                .map(move |delta| {
                    Offset::from_time_zero(
                        (delta + ot.deadline)
                            .since_time_zero()
                            .saturating_sub(tua.deadline),
                    )
                })
                .take_while(|A| *A < max_offset)
        })
        .kmerge()
        .merge(search_space_tua)
        .dedup();

    // Finally, apply the offset-specific RTA to each offset in the
    // search space and return the maximum response-time bound.
    fixed_point::max_response_time(search_space.map(rta))
}
