use crate::arrival::ArrivalBound;
use crate::demand::{self, RequestBound};
use crate::fixed_point;
use crate::supply;
use crate::time::{Duration, Offset, Service};
use crate::wcet;

/// Try to find a response-time bound for a task under fixed-priority
/// scheduling with floating non-preemptive regions on a dedicated
/// uniprocessor.
///
/// The analysis assumes that all tasks are independent and that each
/// is characterized by an arbitrary arrival curve and a WCET bound.
/// The total higher-or-equal-priority interference is represented by
/// `interference`; the task under analysis is given by
/// `task_under_analysis_wcet` and `task_under_analysis_arrivals`;
/// `blocking_bound` is a bound on the maximum priority inversion
/// caused by tasks of lower priority.
///
/// If no fixed point is found below the divergence limit given by
/// `limit`, return a [SearchFailure][fixed_point::SearchFailure]
/// instead.
///
/// This analysis is an implementation of the corresponding  verified
/// instantiation of [the abstract RTA of Bozhko and Brandenburg
/// (ECRTS 2020)](https://drops.dagstuhl.de/opus/volltexte/2020/12385/pdf/LIPIcs-ECRTS-2020-22.pdf).
/// See also [the Coq-verified instantiation](http://prosa.mpi-sws.org/branches/master/pretty/prosa.results.fixed_priority.rta.floating_nonpreemptive.html).
#[allow(non_snake_case)]
pub fn dedicated_uniproc_rta<RBF, AB>(
    interference: &RBF,
    task_under_analysis_wcet: &wcet::Scalar,
    task_under_analysis_arrivals: &AB,
    blocking_bound: Service,
    limit: Duration,
) -> fixed_point::SearchResult
where
    RBF: RequestBound + ?Sized,
    AB: ArrivalBound + ?Sized,
{
    // This analysis is specific to dedicated uniprocessors.
    let proc = supply::Dedicated::new();

    // For convenience, define the RBF for the task under analysis.
    let task_under_analysis =
        demand::RBF::new(&task_under_analysis_arrivals, &task_under_analysis_wcet);

    // First, bound the maximum possible busy-window length.
    let L = fixed_point::search(&proc, limit, |L| {
        blocking_bound + interference.service_needed(L) + task_under_analysis.service_needed(L)
    })?;

    // Second, define the RTA for a given offset A. To this end, we
    // define some trivial components of the fixed-point equation to
    // implement the RTA given in the aRTA paper as literally as
    // possible.

    // The run-to-completion threshold of the task under analysis, In
    // the model with floating non-preemptive regions, there is no
    // static information about the placement of preemption points in
    // all jobs, i.e., it is impossible to predict when exactly a job
    // will be preemptable. Thus, the only safe run-to-completion
    // threshold is the task's WCET.
    // See also: http://prosa.mpi-sws.org/branches/master/pretty/prosa.model.task.preemption.floating_nonpreemptive.html
    let rtct = task_under_analysis_wcet.wcet;
    // The remaining cost after the run-to-completion threshold has been reached.
    // (So obviously it is zero, which the compiler will figure out.)
    let rem_cost = task_under_analysis_wcet.wcet - rtct;

    // Now define the offset-specific RTA.
    let rta = |A: Offset| {
        // Define the RHS of the equation in theorem 31 of the aRTA paper,
        // where AF = A + F.
        let rhs = |AF: Duration| {
            // demand of the task under analysis
            let self_interference = task_under_analysis.service_needed(A.closed_since_time_zero());
            let tua_demand = self_interference - rem_cost;

            // demand of all interfering tasks
            let interfering_demand = interference.service_needed(AF);

            // considering `blocking_bound` to account for priority inversion
            blocking_bound + tua_demand + interfering_demand
        };

        // Find the solution A+F that is the least fixed point.
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
    let search_space = demand::step_offsets(&task_under_analysis).take_while(|A| *A < max_offset);

    // Apply the offset-specific RTA to each offset in the search space and
    // return the maximum response-time bound.
    fixed_point::max_response_time(search_space.map(rta))
}
