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
        isSelected ? "border-l-accent bg-bg-raised" : "border-l-transparent"
      )}
    >
      <div className="flex items-center gap-2">
        <SeverityBadge severity={finding.severity} />
        <span className="text-[13px] font-semibold text-text-primary truncate">
          {finding.title}
        </span>
      </div>
      <p className="text-[11px] text-text-muted leading-relaxed mt-2 pl-0.5">
        {finding.detail}
      </p>
      {finding.evidence_url && (
        <a
          href={finding.evidence_url}
          target="_blank"
          rel="noopener noreferrer"
          onClick={(e) => e.stopPropagation()}
          className="text-[11px] text-accent hover:underline mt-1.5 inline-block pl-0.5"
        >
          evidence ↗
        </a>
      )}
    </button>
  );
}
