import clsx from "clsx";
import type { Finding } from "../../types";
import { SeverityBadge } from "../shared/SeverityBadge";
import { useInvestigation } from "../../context/useInvestigation";

export function FindingCard({ finding }: { finding: Finding }) {
  const { selectedFindingId, selectFinding } = useInvestigation();
  const isSelected = selectedFindingId === finding.id;
  const hasDiff = finding.declared_value || finding.actual_value;

  return (
    <button
      onClick={() => selectFinding(finding.id)}
      className={clsx(
        "w-full text-left px-4 py-3 border-l-3 transition-all duration-150 cursor-pointer",
        "hover:bg-bg-raised",
        isSelected ? "border-l-accent bg-bg-raised" : "border-l-transparent"
      )}
    >
      <div className="flex items-center gap-2">
        <SeverityBadge severity={finding.severity} />
        <span className="text-[13px] font-semibold text-text-primary">
          {finding.title}
        </span>
      </div>
      <p className="text-[11px] text-text-secondary leading-relaxed mt-2 pl-0.5">
        {finding.detail}
      </p>
      {hasDiff && (
        <div className="mt-2 ml-0.5 text-[10px] font-mono rounded border border-border bg-bg-base px-2.5 py-1.5">
          {finding.declared_value && (
            <div className="flex gap-2">
              <span className="text-severity-high w-12 shrink-0">declared</span>
              <span className="text-text-primary">{finding.declared_value}</span>
            </div>
          )}
          {finding.actual_value && (
            <div className="flex gap-2">
              <span className="text-status-ok w-12 shrink-0">actual</span>
              <span className="text-text-primary">{finding.actual_value}</span>
            </div>
          )}
        </div>
      )}
      {finding.reason && (
        <p className="text-[10px] text-text-muted leading-relaxed mt-1.5 pl-0.5 italic">
          {finding.reason}
        </p>
      )}
      {finding.evidence_url && (
        <a
          href={finding.evidence_url}
          target="_blank"
          rel="noopener noreferrer"
          onClick={(e) => e.stopPropagation()}
          className="text-[11px] text-link hover:underline mt-1.5 inline-block pl-0.5"
        >
          evidence ↗
        </a>
      )}
    </button>
  );
}
