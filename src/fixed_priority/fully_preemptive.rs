use crate::demand::{self, RequestBound};
use crate::time::{Duration, Offset, Service};
use crate::{fixed_point, supply};

/// Try to find a response-time bound for a task under
/// fully-preemptive fixed-priority scheduling on a dedicated
/// uniprocessor.
///
/// The analysis assumes that all tasks are independent and that each
/// is characterized by an arbitrary arrival curve and a WCET bound.
/// The total higher-or-equal-priority interference is represented by
/// `interfering_tasks`, the task under analysis is given by `tua`.
///
/// If no fixed point is found below the divergence limit given by
/// `limit`, the function returns a
/// [SearchFailure][fixed_point::SearchFailure] instead.
///
/// This analysis is an implementation of the corresponding  verified
/// instantiation of [the abstract RTA of Bozhko and Brandenburg
/// (ECRTS 2020)](https://drops.dagstuhl.de/opus/volltexte/2020/12385/pdf/LIPIcs-ECRTS-2020-22.pdf).
/// See also [the Coq-verified instantiation](http://prosa.mpi-sws.org/branches/master/pretty/prosa.results.fixed_priority.rta.fully_preemptive.html).
#[allow(non_snake_case)]
pub fn dedicated_uniproc_rta<TaskUnderAnalysisRBF, InterferingRBF>(
    tua: &TaskUnderAnalysisRBF,
    interfering_tasks: &[InterferingRBF],
    limit: Duration,
) -> fixed_point::SearchResult
where
    TaskUnderAnalysisRBF: RequestBound + ?Sized,
    InterferingRBF: RequestBound,
{
    // This analysis is specific to dedicated uniprocessors.
    let proc = supply::Dedicated::new();

    // First, bound the maximum possible busy-window length.
    let L = fixed_point::search(&proc, limit, |L| {
        let interference_bound: Service = interfering_tasks
            .iter()
            .map(|rbf| rbf.service_needed(L))
            .sum();
        interference_bound + tua.service_needed(L)
    })?;

    // Second, define the RTA for a given offset A.
    let rta = |A: Offset| {
        // Define the RHS of the equation in theorem 31 of the aRTA paper,
        // where AF = A + F.
        let rhs = |AF: Duration| {
            // demand of the task under analysis
            let tua_demand = tua.service_needed(A.closed_since_time_zero());

            // demand of all interfering tasks
            let interfering_demand = interfering_tasks
                .iter()
                .map(|rbf| rbf.service_needed(AF))
                .sum();

            tua_demand + interfering_demand
        };

        // Find the solution A+F that is the least fixed point.
        let AF = fixed_point::search(&proc, limit, rhs)?;
        // Extract the corresponding bound.
        let F = AF - A.since_time_zero();
        Ok(F)
    };

    // Third, define the search space. The search space is given by
    // A=0 and each step below L of the task under analysis's RBF.
    // The case of A=0 is not handled explicitly since `step_offsets()`
    // necessarily yields it.
    let max_offset = Offset::from_time_zero(L);
    let search_space = demand::step_offsets(tua).take_while(|A| *A < max_offset);

    // Apply the offset-specific RTA to each offset in the search space and
    // return the maximum response-time bound.
    fixed_point::max_response_time(search_space.map(rta))
}
