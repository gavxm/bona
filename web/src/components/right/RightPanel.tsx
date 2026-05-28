import { useInvestigation } from "../../context/useInvestigation";
import { FindingsHeader } from "./FindingsHeader";
import { FindingCard } from "./FindingCard";

export function RightPanel() {
  const { investigation } = useInvestigation();
  if (!investigation) return null;

  const { findings } = investigation;

  return (
    <div className="h-full overflow-y-auto bg-bg-surface">
      <FindingsHeader findings={findings} />
      {findings.length === 0 ? (
        <EmptyFindings />
      ) : (
        <div className="divide-y divide-border">
          {findings.map((f) => (
            <FindingCard key={f.id} finding={f} />
          ))}
        </div>
      )}
    </div>
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
    <div className="px-4 py-4">
      <p className="text-xs text-status-ok mb-3">No issues found.</p>
      <p className="text-[11px] text-text-muted mb-2">checked:</p>
      {checks.map((check) => (
        <div key={check} className="flex items-center gap-2 py-0.5">
          <span className="text-status-ok text-[11px]">✓</span>
          <span className="text-[11px] text-text-secondary">{check}</span>
        </div>
      ))}
    </div>
  );
}
