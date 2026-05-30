export type CenterTab = "declared" | "config" | "community" | "sources";

export interface FindingLink {
  graphNodes: ("parent" | "subject")[];
  centerTab: CenterTab;
  centerFields: string[];
}

export const FINDING_LINKS: Record<string, FindingLink> = {
  license_inheritance_violation: {
    graphNodes: ["parent", "subject"],
    centerTab: "declared",
    centerFields: ["declared_license"],
  },
  license_mismatch: {
    graphNodes: ["parent", "subject"],
    centerTab: "declared",
    centerFields: ["declared_license"],
  },
  license_unverifiable: {
    graphNodes: ["parent", "subject"],
    centerTab: "declared",
    centerFields: ["declared_license"],
  },
  lineage_inconsistency: {
    graphNodes: ["parent", "subject"],
    centerTab: "config",
    centerFields: ["model_type", "architectures"],
  },
  missing_license: {
    graphNodes: ["subject"],
    centerTab: "declared",
    centerFields: ["declared_license"],
  },
  missing_base_model: {
    graphNodes: ["subject"],
    centerTab: "declared",
    centerFields: ["declared_base_model"],
  },
  new_account: {
    graphNodes: [],
    centerTab: "community",
    centerFields: ["author_created_at"],
  },
  no_community_activity: {
    graphNodes: [],
    centerTab: "community",
    centerFields: ["discussion_count"],
  },
  model_type_not_in_tags: {
    graphNodes: [],
    centerTab: "config",
    centerFields: ["model_type"],
  },
  weight_size_anomaly: {
    graphNodes: [],
    centerTab: "config",
    centerFields: ["safetensors_total_size"],
  },
  gated_derivative: {
    graphNodes: ["parent", "subject"],
    centerTab: "declared",
    centerFields: ["declared_license"],
  },
  transitive_license_violation: {
    graphNodes: ["parent", "subject"],
    centerTab: "declared",
    centerFields: ["declared_license"],
  },
  low_engagement: {
    graphNodes: [],
    centerTab: "declared",
    centerFields: ["downloads", "likes"],
  },
  recently_modified: {
    graphNodes: [],
    centerTab: "declared",
    centerFields: ["created_at", "last_modified"],
  },
  undeclared_quantization: {
    graphNodes: [],
    centerTab: "config",
    centerFields: ["quant_method"],
  },
  no_weight_files: {
    graphNodes: ["subject"],
    centerTab: "declared",
    centerFields: ["files"],
  },
  suspicious_files: {
    graphNodes: ["subject"],
    centerTab: "declared",
    centerFields: ["files"],
  },
};
