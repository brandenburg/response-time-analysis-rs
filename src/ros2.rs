use crate::analysis::{self, RequestBound};
use crate::supply::SupplyBound;
use crate::time::*;

pub fn rta_event_source<SBF, RBF>(supply: SBF, demand: RBF, limit: Duration) -> Option<Duration>
where
    SBF: SupplyBound,
    RBF: RequestBound,
{
    let offset = 0;
    // right-hand side of Lemma 1
    let rhs = |_delta| demand.service_needed(offset + 1);
    analysis::fixed_point_search(supply, offset, limit, rhs)
}
