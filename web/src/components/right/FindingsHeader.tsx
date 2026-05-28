import type { Finding } from "../../types";

export function FindingsHeader({ findings }: { findings: Finding[] }) {
  const counts = { high: 0, medium: 0, low: 0, info: 0 };
  for (const f of findings) counts[f.severity]++;

  const parts: string[] = [];
  if (counts.high > 0) parts.push(`${counts.high} high`);
  if (counts.medium > 0) parts.push(`${counts.medium} medium`);
  if (counts.low > 0) parts.push(`${counts.low} low`);
  if (counts.info > 0) parts.push(`${counts.info} info`);

  return (
    <div className="px-3 py-3 border-b border-border">
      <h2 className="text-xs font-semibold text-text-secondary uppercase tracking-wide">
        Findings
      </h2>
      {findings.length > 0 && (
        <p className="text-[11px] text-text-muted mt-0.5">
          {findings.length} finding{findings.length !== 1 ? "s" : ""} ({parts.join(", ")})
        </p>
      )}
    </div>
  );
}
