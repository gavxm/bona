import { useInvestigation } from "../../context/useInvestigation";
import { StatusDot } from "../shared/StatusDot";

const SOURCE_LABELS: Record<string, string> = {
  hf_metadata: "HF metadata",
  model_tree: "model tree",
  model_config: "model config",
  community_signals: "community",
};

export function SourcesTab() {
  const { investigation } = useInvestigation();
  if (!investigation) return null;

  return (
    <div className="py-2">
      {investigation.sources.map((rec) => (
        <div
          key={rec.source}
          className="flex items-center gap-3 px-4 py-1.5"
        >
          <StatusDot status={rec.status} />
          <span className="text-text-secondary text-xs w-32">
            {SOURCE_LABELS[rec.source] ?? rec.source}
          </span>
          <span className="text-xs font-mono text-text-muted">
            {rec.status.status === "ok"
              ? `${rec.status.fetched_ms}ms`
              : rec.status.status === "failed"
                ? rec.status.reason
                : "not implemented"}
          </span>
        </div>
      ))}
    </div>
  );
}
