import type { Finding } from "../../types";

export function FindingsHeader({ findings }: { findings: Finding[] }) {
  return (
    <div className="px-5 py-3 border-b border-border flex items-center gap-2.5">
      <span className="font-mono text-[11px] font-semibold tracking-widest uppercase text-text-secondary">
        Findings
      </span>
      <span className="font-mono text-[10.5px] text-text-muted">
        {findings.length} detected
      </span>
    </div>
  );
}
