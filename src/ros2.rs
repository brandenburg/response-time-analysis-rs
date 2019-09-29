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
    let rhs = |offset, _delta| demand.service_needed(offset + 1);
    analysis::response_time_analysis(supply, demand, rhs_busy_window, rhs, limit)
}
