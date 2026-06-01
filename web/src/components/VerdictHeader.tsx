import { useInvestigation } from "../context/useInvestigation";
import { IconAlert, IconVerifiedSeal } from "./shared/Icons";
import type { Finding } from "../types";

const RELATION_LABELS: Record<string, string> = {
  finetune: "fine-tune",
  quantization: "quantized",
  merge: "merge",
  adapter: "adapter",
};

function formatTimestamp(iso: string): string {
  try {
    const dt = new Date(iso);
    return dt.toLocaleString("en", {
      month: "short",
      day: "numeric",
      year: "numeric",
      hour: "2-digit",
      minute: "2-digit",
    });
  } catch {
    return iso;
  }
}

/** Build a readable summary sentence from the findings. */
function buildSummary(findings: Finding[], _modelId: string, parentId?: string | null): string | null {
  const high = findings.filter((f) => f.severity === "high");
  const med = findings.filter((f) => f.severity === "medium");
  const significant = [...high, ...med];
  if (significant.length === 0) return null;

  const parts: string[] = [];

  // Group by category for more natural prose.
  const hasLicense = significant.some((f) => f.id.startsWith("license") || f.id === "transitive_license_violation");
  const hasGated = significant.some((f) => f.id === "gated_derivative");
  const hasLineage = significant.some((f) => f.id === "lineage_inconsistency");
  const hasSuspiciousFiles = significant.some((f) => f.id === "suspicious_files" || f.id === "no_weight_files");
  const hasQuant = significant.some((f) => f.id === "undeclared_quantization" || f.id === "weight_size_anomaly");
  const hasTrust = significant.some((f) => f.id === "new_account" || f.id === "recently_modified");

  if (hasLicense && parentId) {
    const parentShort = parentId.split("/").pop() ?? parentId;
    const declaredLicense = findings.find((f) => f.id.startsWith("license"))?.declared_value;
    if (declaredLicense) {
      parts.push(`declares ${declaredLicense} but derives from ${parentShort}, whose license must propagate`);
    } else {
      parts.push(`license conflicts with ${parentShort}`);
    }
  } else if (hasLicense) {
    parts.push("license inheritance issues detected");
  }

  if (hasGated) parts.push("redistributes weights from a gated origin");
  if (hasLineage) parts.push("declared lineage does not match detected origin");
  if (hasSuspiciousFiles) parts.push("contains suspicious or missing weight files");
  if (hasQuant) parts.push("weight size does not match declared precision");
  if (hasTrust) parts.push("trust signals indicate elevated risk");

  if (parts.length === 0) {
    // Fallback: just list titles.
    return significant.map((f) => f.title.toLowerCase()).join(", ") + ".";
  }

  // Capitalize first part, join with semicolons for readability.
  const sentence = parts.join("; ");
  return sentence.charAt(0).toUpperCase() + sentence.slice(1) + ".";
}

export function VerdictHeader() {
  const { investigation, isSnapshot, loadInvestigation } = useInvestigation();
  if (!investigation) return null;

  const findings = investigation.findings;
  const highCount = findings.filter((f) => f.severity === "high").length;
  const medCount = findings.filter((f) => f.severity === "medium").length;
  const lowCount = findings.filter((f) => f.severity === "low").length;
  const totalCount = findings.length;

  const sourcesOk = investigation.sources.filter((s) => s.status.status === "ok").length;
  const sourcesTotal = investigation.sources.length;

  const isClean = highCount === 0 && medCount === 0;
  const isHigh = highCount > 0;

  const statusLabel = isClean ? "Clean" : isHigh ? "Untrusted" : "Caution";
  const statusColor = isClean
    ? "text-status-ok"
    : isHigh
      ? "text-severity-high"
      : "text-severity-medium";

  const parentId = investigation.lineage?.chain[0]?.model_id;
  const why = buildSummary(findings, investigation.model_id, parentId);

  const relation = investigation.declared.base_model_relation;
  const typeLabel = relation && relation !== "unknown" ? RELATION_LABELS[relation] ?? relation : null;

  const issueParts: string[] = [];
  if (highCount > 0) issueParts.push(`${highCount} high`);
  if (medCount > 0) issueParts.push(`${medCount} med`);
  if (lowCount > 0) issueParts.push(`${lowCount} low`);

  return (
    <div
      className="flex flex-col lg:flex-row lg:items-stretch border-b border-border shrink-0"
      style={{ background: "linear-gradient(180deg, var(--color-bg-surface) 0%, var(--color-bg-base) 100%)" }}
    >
      {/* Headline block */}
      <div className="flex-1 min-w-0 px-4 lg:px-6 py-4">
        <div className="flex items-center gap-2.5 flex-wrap">
          <span className="text-xl font-semibold tracking-tight text-text-primary">
            {investigation.model_id}
          </span>
          {typeLabel && (
            <span className="font-mono text-[10.5px] text-text-muted border border-border-strong px-1.5 py-0.5 rounded">
              {typeLabel}
            </span>
          )}
        </div>
        <div className={`flex items-center gap-1.5 mt-2 text-[13px] font-semibold uppercase tracking-widest ${statusColor}`}>
          {isClean ? <IconVerifiedSeal size={16} /> : <IconAlert size={14} className={statusColor} />}
          {statusLabel}
        </div>
        {why && (
          <p className="mt-1.5 text-[13.5px] text-text-secondary leading-relaxed max-w-2xl" style={{ textWrap: "pretty" } as React.CSSProperties}>
            {why}
          </p>
        )}
        {isClean && (
          <p className="mt-1.5 text-[13.5px] text-text-secondary leading-relaxed">
            All evidence sources checked. No license, lineage, or trust issues detected.
          </p>
        )}
      </div>

      {/* Stats + rescan */}
      <div className="flex items-center px-4 pb-3 gap-4 lg:px-0 lg:pb-0 lg:gap-0">
        <Stat label="issues" value={totalCount.toString()} sub={issueParts.join(" - ")} bad={!isClean} />
        <Stat label="sources" value={`${sourcesOk}/${sourcesTotal}`} />
        <Stat label="investigated" value={formatTimestamp(investigation.investigated_at)} />
        {isSnapshot && (
          <div className="px-5 py-4 border-l border-border flex items-center">
            <button
              onClick={() => loadInvestigation(investigation.model_id)}
              className="h-[30px] px-3 rounded-lg border border-border-strong bg-bg-raised text-text-secondary text-[11px] font-medium cursor-pointer hover:text-text-primary hover:bg-bg-open transition-colors"
            >
              re-scan
            </button>
          </div>
        )}
      </div>
    </div>
  );
}

function Stat({ label, value, sub, bad }: { label: string; value: string; sub?: string; bad?: boolean }) {
  return (
    <div className="px-5 py-4 border-l border-border flex flex-col gap-1 items-start">
      <span className="font-mono text-[10px] tracking-widest uppercase text-text-muted">{label}</span>
      <span className={`text-base font-semibold tabular-nums ${bad ? "text-severity-high" : "text-text-primary"}`}>
        {value}
        {sub && <span className="text-xs text-text-muted font-normal ml-1.5">{sub}</span>}
      </span>
    </div>
  );
}
