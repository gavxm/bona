import { useInvestigation } from "../context/useInvestigation";

export function SummaryStrip() {
  const { investigation } = useInvestigation();
  if (!investigation) return null;

  const findings = investigation.findings;
  const highCount = findings.filter((f) => f.severity === "high").length;
  const medCount = findings.filter((f) => f.severity === "medium").length;
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

  return (
    <div className="flex items-center gap-4 px-4 py-1.5 border-b border-border bg-bg-surface text-[11px]">
      <span className="text-text-muted">verdict:</span>
      {isClean ? (
        <span className="text-status-ok font-medium">no critical issues</span>
      ) : (
        <span className="text-severity-high font-medium">
          {highCount > 0 && `${highCount} high`}
          {highCount > 0 && medCount > 0 && ", "}
          {medCount > 0 && `${medCount} medium`}
          {" "}finding{highCount + medCount !== 1 ? "s" : ""}
        </span>
      )}
      <span className="text-text-muted">·</span>
      <span className="text-text-secondary">
        {sourcesOk}/{sourcesOk + sourcesFailed} sources
      </span>
      <span className="text-text-muted">·</span>
      <span className="text-text-muted font-mono">{totalTime}ms</span>
    </div>
  );
}
