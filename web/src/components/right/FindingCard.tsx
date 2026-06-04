import type { Finding, Severity } from "../../types";
import { FINDING_LINKS } from "../../linking";
import { useInvestigation } from "../../context/useInvestigation";
import { IconChevronDown, IconArrowRight } from "../shared/Icons";

const SEV_COLORS: Record<Severity, { text: string; bg: string; line: string }> = {
  high: { text: "text-severity-high", bg: "bg-severity-high-bg", line: "border-severity-high-line" },
  medium: { text: "text-severity-medium", bg: "bg-severity-medium-bg", line: "border-severity-medium-line" },
  low: { text: "text-severity-low", bg: "bg-severity-low-bg", line: "border-severity-low-line" },
  info: { text: "text-text-muted", bg: "bg-bg-raised", line: "border-border" },
};

export function FindingCard({ finding, open, related }: { finding: Finding; open: boolean; related: boolean }) {
  const { selectFinding, setActiveTab } = useInvestigation();
  const colors = SEV_COLORS[finding.severity];
  const link = FINDING_LINKS[finding.id];

  const jumpTo = (tab: string) => {
    setActiveTab(tab as "declared" | "config" | "community" | "sources");
    // pulseKey is already bumped by selectFinding; re-bumping here would need a separate mechanism.
    // For now, clicking the chip just switches the tab.
  };

  return (
    <div
      className={[
        "border rounded-[10px] overflow-hidden transition-all duration-200 mb-2 cursor-pointer",
        open
          ? `border-${finding.severity === "high" ? "severity-high-line" : finding.severity === "medium" ? "severity-medium-line" : "border-strong"} bg-bg-open shadow-[0_1px_2px_rgba(0,0,0,0.4),0_8px_24px_rgba(0,0,0,0.28)]`
          : "border-border bg-bg-raised hover:border-border-strong",
        related && !open
          ? "border-accent-line shadow-[0_0_0_1px_var(--color-accent-line)]"
          : "",
      ].join(" ")}
    >
      {/* Head */}
      <div
        className="flex items-start gap-3 px-3.5 py-3"
        onClick={() => selectFinding(finding.id)}
      >
        {/* Severity bar */}
        <span
          className={`shrink-0 w-0.75 self-stretch rounded-full ${colors.text.replace("text-", "bg-")}`}
        />

        {/* Content */}
        <div className="flex-1 min-w-0">
          <div className="text-[13.5px] font-semibold text-text-primary tracking-tight">
            {finding.title}
          </div>
          {finding.detail && (
            <div className="text-[12px] text-text-secondary mt-1 leading-relaxed line-clamp-2">
              {open ? null : finding.detail}
            </div>
          )}
          <div className="flex items-center gap-2 mt-2">
            <span
              className={`font-mono text-[9px] tracking-wide uppercase font-semibold px-1.5 py-0.5 rounded border ${colors.text} ${colors.bg} ${colors.line}`}
            >
              {finding.severity}
            </span>
            <span className="font-mono text-[9.5px] text-text-muted">
              {finding.id}
            </span>
          </div>
        </div>

        {/* Chevron */}
        <span
          className={`shrink-0 text-text-muted mt-0.5 transition-transform duration-200 ${open ? "rotate-180" : ""}`}
        >
          <IconChevronDown size={15} />
        </span>
      </div>

      {/* Expanded detail */}
      {open && (
        <div className="px-3.5 pb-3.5 pl-7">
          {/* Description */}
          <div className="mt-1">
            <p
              className="text-[12.5px] leading-relaxed text-text-secondary"
              style={{ textWrap: "pretty" } as React.CSSProperties}
            >
              {finding.detail}
            </p>
          </div>

          {/* Evidence chips */}
          {link && link.centerFields.length > 0 && (
            <div className="mt-3">
              <div className="font-mono text-[9.5px] tracking-widest uppercase text-text-muted mb-1.5">
                implicated evidence
              </div>
              <div className="flex flex-wrap gap-1.5">
                {link.centerFields.map((field) => (
                  <span
                    key={field}
                    onClick={(e) => {
                      e.stopPropagation();
                      jumpTo(link.centerTab);
                    }}
                    className="font-mono text-[10.5px] text-accent-text bg-accent-bg border border-accent-line px-2 py-0.5 rounded-md inline-flex items-center gap-1.5 cursor-pointer hover:border-accent transition-colors"
                  >
                    {link.centerTab} - {field}
                    <IconArrowRight size={9} className="opacity-70" />
                  </span>
                ))}
              </div>
            </div>
          )}

          {/* Recommendation */}
          {finding.reason && (
            <div className="mt-3">
              <div className="font-mono text-[9.5px] tracking-widest uppercase text-text-muted mb-1.5">
                recommendation
              </div>
              <div
                className={`text-[12.5px] leading-relaxed text-text-primary px-3 py-2.5 rounded-lg border ${colors.bg} ${colors.line}`}
              >
                {finding.reason}
              </div>
            </div>
          )}

          {/* Evidence link */}
          {finding.evidence_url && (
            <a
              href={finding.evidence_url}
              target="_blank"
              rel="noopener noreferrer"
              onClick={(e) => e.stopPropagation()}
              className="text-[11px] text-link hover:underline mt-2 inline-block"
            >
              view on HuggingFace
            </a>
          )}
        </div>
      )}
    </div>
  );
}
