pub mod community;
pub mod hf_metadata;
pub mod model_config;
pub mod model_tree;

use crate::{BonaError, EvidenceSource, SourceRecord, SourceStatus};

/// The result of fetching a single evidence source.
pub struct FetchResult {
    pub record: SourceRecord,
    pub evidence: Option<Evidence>,
}

/// Source-specific evidence data, merged into the investigation after all
/// sources complete.
#[allow(dead_code)] // Variants constructed as sources are implemented.
pub enum Evidence {
    HfMetadata(hf_metadata::HfMetadataEvidence),
    ModelTree(model_tree::ModelTreeEvidence),
    ModelConfig(model_config::ModelConfigEvidence),
    Community(community::CommunityEvidence),
}

impl FetchResult {
    /// Convenience for sources that aren't implemented yet.
    pub fn not_implemented(source: EvidenceSource) -> Self {
        FetchResult {
            record: SourceRecord {
                source,
                status: SourceStatus::NotImplemented,
            },
            evidence: None,
        }
    }

    /// Convenience for a source that failed.
    pub fn failed(source: EvidenceSource, err: BonaError) -> Self {
        FetchResult {
            record: SourceRecord {
                source,
                status: SourceStatus::Failed {
                    reason: err.to_string(),
                },
            },
            evidence: None,
        }
    }

    /// Convenience for a source that succeeded.
    pub fn ok(source: EvidenceSource, fetched_ms: u64, evidence: Evidence) -> Self {
        FetchResult {
            record: SourceRecord {
                source,
                status: SourceStatus::Ok { fetched_ms },
            },
            evidence: Some(evidence),
        }
    }
}
