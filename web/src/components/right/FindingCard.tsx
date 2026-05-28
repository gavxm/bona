import clsx from "clsx";
import type { Finding } from "../../types";
import { SeverityBadge } from "../shared/SeverityBadge";
import { useInvestigation } from "../../context/useInvestigation";

export function FindingCard({ finding }: { finding: Finding }) {
  const { selectedFindingId, selectFinding } = useInvestigation();
  const isSelected = selectedFindingId === finding.id;

  return (
    <button
      onClick={() => selectFinding(finding.id)}
      className={clsx(
        "w-full text-left px-4 py-3 border-l-3 transition-all duration-150 cursor-pointer",
        "hover:bg-bg-raised",
        isSelected
          ? "border-l-accent bg-bg-raised"
          : "border-l-transparent"
      )}
    >
      <div className="flex items-center gap-2 mb-1">
        <SeverityBadge severity={finding.severity} />
        <span className="text-sm font-medium text-text-primary truncate">
          {finding.title}
        </span>
      </div>
      <p className="text-xs text-text-secondary leading-relaxed">
        {finding.detail}
      </p>
      {finding.evidence_url && (
        <a
          href={finding.evidence_url}
          target="_blank"
          rel="noopener noreferrer"
          onClick={(e) => e.stopPropagation()}
          className="text-xs text-accent hover:underline mt-1 inline-block"
        >
          evidence ↗
        </a>
      )}
    </button>
  );
}
