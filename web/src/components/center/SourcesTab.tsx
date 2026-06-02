import { useInvestigation } from "../../context/useInvestigation";

const SOURCE_INFO: Record<string, { name: string; meta: string }> = {
  hf_metadata: { name: "Hub model metadata", meta: "API card, license, tags, files" },
  model_tree: { name: "Model tree", meta: "lineage chain, siblings" },
  model_config: { name: "config.json", meta: "architecture, parameters, quantization" },
  community_signals: { name: "Community & discussions", meta: "account age, engagement, discussions" },
};

export function SourcesTab() {
  const { investigation } = useInvestigation();
  if (!investigation) return null;

  return (
    <div className="py-2">
      {investigation.sources.map((rec) => {
        const info = SOURCE_INFO[rec.source] ?? { name: rec.source, meta: "" };
        const isOk = rec.status.status === "ok";
        const statusText = rec.status.status === "ok"
          ? `fetched ${rec.status.fetched_ms}ms`
          : rec.status.status === "failed"
            ? rec.status.reason
            : "not implemented";

        return (
          <div
            key={rec.source}
            className="flex items-center gap-3 py-3 border-t border-border first:border-t-0"
          >
            <span className={`w-2.5 h-2.5 rounded-full shrink-0 ${isOk ? "bg-status-ok" : "bg-severity-medium"}`} />
            <div className="flex-1 min-w-0">
              <div className="text-[13px] font-medium text-text-primary">{info.name}</div>
              <div className="font-mono text-[10.5px] text-text-muted mt-0.5">{info.meta}</div>
            </div>
            <span className={`font-mono text-[10.5px] shrink-0 ${isOk ? "text-status-ok" : "text-severity-medium"}`}>
              {statusText}
            </span>
          </div>
        );
      })}
    </div>
  );
}
