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
  declared_value?: string | null;
  actual_value?: string | null;
  evidence_url: string | null;
}

export type RelationKind =
  | "finetune"
  | "quantization"
  | "merge"
  | "adapter"
  | "unknown";

export interface DeclaredFacts {
  model_id: string;
  declared_license: string | null;
  declared_base_model: string | null;
  base_model_relation?: RelationKind | null;
  library: string | null;
  pipeline_tag: string | null;
  tags: string[];
  downloads: number | null;
  gated?: string | null;
  sha?: string | null;
  last_modified?: string | null;
  likes?: number | null;
  private?: boolean | null;
  files?: string[];
  created_at?: string | null;
}

export interface LineageNode {
  model_id: string;
  license: string | null;
  relation: RelationKind;
  exists: boolean;
  gated?: string | null;
  depth: number;
  error?: string | null;
}

export interface LineageEvidence {
  chain: LineageNode[];
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
  quant_method?: string | null;
  quant_bits?: number | null;
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
  investigated_at: string;
  declared: DeclaredFacts;
  lineage: LineageEvidence | null;
  config: ModelConfigEvidence | null;
  community: CommunityEvidence | null;
  sources: SourceRecord[];
  findings: Finding[];
}
