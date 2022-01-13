use crate::demand::{self, RequestBound};
use crate::time::{Duration, Offset};
use crate::{fixed_point, supply};

/// Try to find a response-time bound for any task in a task set
/// under FIFO scheduling on a dedicated uniprocessor.
///
/// The analysis assumes that all tasks are independent and that each
/// is characterized by an arbitrary arrival curve and a WCET bound.
/// The total rbf is represented by `tasks_rbf`
///
/// Note that all tasks share the same response-time bound under FIFO
/// scheduling; hence this function does not take any parameters
/// specific to a "task under analysis."
///
/// If no fixed point is found below the divergence limit given by
/// `limit`, return a [SearchFailure][fixed_point::SearchFailure]
/// instead.
#[allow(non_snake_case)]
pub fn dedicated_uniproc_rta<RBF>(tasks_rbf: &RBF, limit: Duration) -> fixed_point::SearchResult
where
    RBF: RequestBound + ?Sized,
{
    // This analysis is specific to dedicated uniprocessors.
    let proc = supply::Dedicated::new();

    // First, bound the maximum possible busy-window length.
    let L = fixed_point::search(&proc, limit, |L| tasks_rbf.service_needed(L))?;

    // Now define the offset-specific RTA.
    let rta = |A: Offset| {
        // Demand of all tasks
        let total_service = tasks_rbf.service_needed(A.closed_since_time_zero());
        // Extract the corresponding bound.
        Duration::from(total_service) - A.since_time_zero()
    };

    // Third, define the search space. The search space is given by
    // A=0 and each step below L of the task under analysis's RBF.
    // The case of A=0 is not handled explicitly since
    // `step_offsets()` necessarily yields it.
    let max_offset = Offset::from_time_zero(L);
    let search_space = demand::step_offsets(&tasks_rbf).take_while(|A| *A < max_offset);

    // Apply the offset-specific RTA to each offset in the search space and
    // return the maximum response-time bound.
    Ok(search_space.map(rta).max().unwrap_or_else(Duration::zero))
}
