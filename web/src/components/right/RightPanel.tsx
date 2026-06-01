import { useState } from "react";
import { useInvestigation } from "../../context/useInvestigation";
import { FindingsHeader } from "./FindingsHeader";
import { FindingCard } from "./FindingCard";
import { IconCheck } from "../shared/Icons";
import type { Severity } from "../../types";

type Filter = "all" | Severity;

const SEV_ORDER: Severity[] = ["high", "medium", "low", "info"];
const SEV_DOT_COLORS: Record<Severity, string> = {
  high: "bg-severity-high",
  medium: "bg-severity-medium",
  low: "bg-severity-low",
  info: "bg-text-muted",
};

export function RightPanel() {
  const { investigation, selectedFindingId, relatedFindings } = useInvestigation();
  const [filter, setFilter] = useState<Filter>("all");

  if (!investigation) return null;

  const { findings } = investigation;
  const counts: Record<Severity, number> = { high: 0, medium: 0, low: 0, info: 0 };
  for (const f of findings) counts[f.severity]++;

  const filtered = filter === "all" ? findings : findings.filter((f) => f.severity === filter);
  const groups = SEV_ORDER.map((sev) => ({
    sev,
    items: filtered.filter((f) => f.severity === sev),
  })).filter((g) => g.items.length > 0);

  return (
    <div className="lg:h-full flex flex-col bg-bg-surface">
      <FindingsHeader findings={findings} />

      {/* Filter pills */}
      {findings.length > 0 && (
        <div className="flex gap-1.5 flex-wrap px-5 py-2.5 border-b border-border">
          <FilterPill label={`all ${findings.length}`} active={filter === "all"} onClick={() => setFilter("all")} />
          {SEV_ORDER.map((sev) =>
            counts[sev] > 0 ? (
              <FilterPill
                key={sev}
                label={`${sev} ${counts[sev]}`}
                active={filter === sev}
                dot={SEV_DOT_COLORS[sev]}
                onClick={() => setFilter(sev)}
              />
            ) : null
          )}
        </div>
      )}

      {/* Findings list */}
      <div className="flex-1 min-h-0 overflow-y-auto scroll">
        <div className="p-5 flex flex-col gap-2">
          {findings.length === 0 ? (
            <EmptyFindings />
          ) : (
            groups.map((g) => (
              <div key={g.sev}>
                {filter === "all" && (
                  <div className="font-mono text-[10px] tracking-widest uppercase text-text-muted mb-2 mt-1 flex items-center gap-2">
                    <span className={`w-2 h-2 rounded-sm ${SEV_DOT_COLORS[g.sev]}`} />
                    {g.sev} <span className="text-text-secondary">- {g.items.length}</span>
                  </div>
                )}
                {g.items.map((f) => (
                  <FindingCard
                    key={f.id}
                    finding={f}
                    open={selectedFindingId === f.id}
                    related={relatedFindings.has(f.id)}
                  />
                ))}
              </div>
            ))
          )}
        </div>
      </div>
    </div>
  );
}

function FilterPill({ label, active, dot, onClick }: { label: string; active: boolean; dot?: string; onClick: () => void }) {
  return (
    <button
      onClick={onClick}
      className={[
        "font-mono text-[10.5px] px-2.5 py-1 rounded-full border cursor-pointer inline-flex items-center gap-1.5 transition-colors",
        active
          ? "bg-accent-bg border-accent-line text-accent-text"
          : "bg-bg-raised border-border-strong text-text-secondary hover:text-text-primary",
      ].join(" ")}
    >
      {dot && <i className={`w-1.5 h-1.5 rounded-full ${dot}`} />}
      {label}
    </button>
  );
}

function EmptyFindings() {
  const checks = [
    "license inheritance",
    "lineage consistency",
    "documentation completeness",
    "trust signals",
    "metadata anomalies",
  ];

  return (
    <div className="py-4">
      <p className="text-xs text-status-ok mb-3">No issues found.</p>
      <p className="text-[11px] text-text-secondary mb-2">All checks passed:</p>
      {checks.map((check) => (
        <div key={check} className="flex items-center gap-2 py-0.5">
          <IconCheck className="text-status-ok shrink-0" />
          <span className="text-[11px] text-text-secondary">{check}</span>
        </div>
      ))}
    </div>
  );
}
