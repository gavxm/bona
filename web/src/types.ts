export type Severity = "info" | "low" | "medium" | "high";

export type EvidenceSource =
  | "hf_metadata"
  | "model_tree"
  | "model_config"
  | "community_signals";

export type SourceStatus =
  | { status: "ok"; fetched_ms: number }
  | { status: "failed"; reason: string }
  | { status: "not_implemented" };

export interface SourceRecord {
  source: EvidenceSource;
  status: SourceStatus;
}

export interface Finding {
  id: string;
  title: string;
  severity: Severity;
  detail: string;
  reason: string;
  evidence_url: string | null;
}

export interface DeclaredFacts {
  model_id: string;
  declared_license: string | null;
  declared_base_model: string | null;
  library: string | null;
  pipeline_tag: string | null;
  tags: string[];
  downloads: number | null;
}

export interface ModelTreeEvidence {
  parent_id: string | null;
  parent_license: string | null;
  parent_exists: boolean | null;
  siblings: string[];
}

export interface ModelConfigEvidence {
  architectures: string[];
  model_type: string | null;
  hidden_size: number | null;
  vocab_size: number | null;
  num_hidden_layers: number | null;
  safetensors_total_size: number | null;
  tokenizer_class: string | null;
}

export interface CommunityEvidence {
  author: string | null;
  author_created_at: string | null;
  author_model_count: number | null;
  discussion_count: number | null;
  closed_discussion_count: number | null;
}

export interface ModelInvestigation {
  schema_version: number;
  model_id: string;
  declared: DeclaredFacts;
  lineage: ModelTreeEvidence | null;
  config: ModelConfigEvidence | null;
  community: CommunityEvidence | null;
  sources: SourceRecord[];
  findings: Finding[];
}
