import { useState } from "react";
import { useInvestigation } from "../context/useInvestigation";
import { ExportModal } from "./ExportModal";

export function SummaryStrip() {
  const { investigation } = useInvestigation();
  const [showExport, setShowExport] = useState(false);

  if (!investigation) return null;

  const findings = investigation.findings;
  const highCount = findings.filter((f) => f.severity === "high").length;
  const medCount = findings.filter((f) => f.severity === "medium").length;
  const lowCount = findings.filter((f) => f.severity === "low").length;
  const sourcesOk = investigation.sources.filter(
    (s) => s.status.status === "ok"
  ).length;
  const sourcesFailed = investigation.sources.filter(
    (s) => s.status.status === "failed"
  ).length;
  const totalTime = investigation.sources.reduce((sum, s) => {
    return sum + (s.status.status === "ok" ? s.status.fetched_ms : 0);
  }, 0);

  const isClean = highCount === 0 && medCount === 0;

  const parts: string[] = [];
  if (highCount > 0) parts.push(`${highCount} high`);
  if (medCount > 0) parts.push(`${medCount} medium`);
  if (lowCount > 0) parts.push(`${lowCount} low`);

  return (
    <>
      <div className="flex items-center justify-between px-4 py-1.5 border-b border-border bg-bg-surface text-[11px]">
        <div className="flex items-center gap-5">
          <div className="flex items-center gap-1.5">
            <span className="text-text-muted">verdict:</span>
            {isClean ? (
              <span className="text-status-ok font-medium">no critical issues</span>
            ) : (
              <span className="text-severity-high font-medium">
                {findings.length} finding{findings.length !== 1 ? "s" : ""} ({parts.join(", ")})
              </span>
            )}
          </div>
          <span className="text-text-muted">·</span>
          <span className="text-text-secondary">
            {sourcesOk}/{sourcesOk + sourcesFailed} sources
          </span>
          <span className="text-text-muted">·</span>
          <span className="text-text-muted font-mono">{totalTime}ms</span>
        </div>
        <button
          onClick={() => setShowExport(true)}
          className="text-text-muted hover:text-text-secondary transition-colors cursor-pointer px-1.5 py-0.5 rounded border border-border hover:border-text-muted text-[10px]"
        >
          export
        </button>
      </div>
      {showExport && <ExportModal onClose={() => setShowExport(false)} />}
    </>
  );
}
