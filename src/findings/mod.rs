//! Cross-referenced finding checks. Each module implements one category of
//! provenance finding, run sequentially against gathered evidence.

mod doc_gap;
mod gated;
mod license;
mod lineage;
mod metadata;
mod trust;

use crate::ModelInvestigation;

/// Run all finding checks against the gathered evidence.
pub fn compute(inv: &mut ModelInvestigation) {
    license::check(inv);
    lineage::check(inv);
    gated::check(inv);
    doc_gap::check(inv);
    trust::check(inv);
    metadata::check(inv);
}
