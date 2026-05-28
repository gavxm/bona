mod license;
mod lineage;

use crate::ModelInvestigation;

/// Run all finding checks against the gathered evidence.
pub fn compute(inv: &mut ModelInvestigation) {
    license::check(inv);
    lineage::check(inv);
}
