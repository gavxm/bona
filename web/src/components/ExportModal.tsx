import { useState, useCallback, useEffect } from "react";
import { useInvestigation } from "../context/useInvestigation";
import { encodePermalink } from "../permalink";
import { IconClose, IconCheck } from "./shared/Icons";

export function ExportModal({ onClose }: { onClose: () => void }) {
  useEffect(() => {
    function handleKey(e: KeyboardEvent) {
      if (e.key === "Escape") onClose();
    }
    document.addEventListener("keydown", handleKey);
    return () => document.removeEventListener("keydown", handleKey);
  }, [onClose]);

  const { investigation, selectedFindingId } = useInvestigation();
  const [copied, setCopied] = useState(false);

  const permalink = investigation
    ? encodePermalink(investigation, selectedFindingId)
    : "";

  const copyPermalink = useCallback(() => {
    navigator.clipboard.writeText(permalink).then(() => {
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    });
  }, [permalink]);

  const downloadJson = useCallback(() => {
    if (!investigation) return;
    const json = JSON.stringify(investigation, null, 2);
    const blob = new Blob([json], { type: "application/json" });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = `${investigation.model_id.replace("/", "--")}.json`;
    a.click();
    URL.revokeObjectURL(url);
  }, [investigation]);

  if (!investigation) return null;

  const findingCount = investigation.findings.length;

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 animate-[fadeIn_150ms_ease-out]"
      onMouseDown={onClose}
    >
      <div
        className="bg-bg-surface border border-border rounded-lg shadow-2xl w-120 max-w-[90vw] animate-[scaleIn_150ms_ease-out]"
        onMouseDown={(e) => e.stopPropagation()}
      >
        {/* header */}
        <div className="flex items-center justify-between px-5 py-4 border-b border-border">
          <h2 className="text-sm font-semibold text-text-primary">
            Export Investigation
          </h2>
          <button
            onClick={onClose}
            className="text-text-muted hover:text-text-primary transition-colors cursor-pointer w-6 h-6 flex items-center justify-center rounded hover:bg-bg-raised"
          >
            <IconClose />
          </button>
        </div>

        {/* body */}
        <div className="px-5 py-4 space-y-5">
          {/* summary */}
          <div className="text-xs text-text-secondary space-y-1">
            <div>
              <span className="text-text-muted w-20 inline-block">model</span>
              <span className="font-mono text-text-primary">
                {investigation.model_id}
              </span>
            </div>
            <div>
              <span className="text-text-muted w-20 inline-block">
                findings
              </span>
              <span className="text-text-primary">
                {findingCount === 0
                  ? "none"
                  : `${findingCount} finding${findingCount !== 1 ? "s" : ""}`}
              </span>
            </div>
          </div>

          {/* permalink */}
          <div>
            <label className="text-[11px] text-text-muted uppercase tracking-wide block mb-1.5">
              Permalink
            </label>
            <div className="flex gap-2">
              <input
                type="text"
                readOnly
                value={permalink}
                className="flex-1 bg-bg-base border border-border rounded px-2.5 py-1.5 text-[11px] font-mono text-text-secondary truncate focus:outline-none focus:border-accent"
                onFocus={(e) => e.target.select()}
              />
              <button
                onClick={copyPermalink}
                className={`px-3 py-1.5 text-xs rounded border transition-colors cursor-pointer shrink-0 flex items-center gap-1.5 ${copied ? "border-status-ok/40 bg-status-ok/10 text-status-ok" : "border-border bg-bg-raised text-text-secondary hover:text-text-primary hover:border-text-muted"}`}
              >
                {copied ? (
                  <>
                    <IconCheck />
                    copied
                  </>
                ) : (
                  "copy"
                )}
              </button>
            </div>
            <p className="text-[10px] text-text-muted mt-1.5">
              Self-contained link with the full investigation encoded in the
              URL.
              {selectedFindingId && " Includes the currently selected finding."}
            </p>
          </div>

          {/* download */}
          <div>
            <label className="text-[11px] text-text-muted uppercase tracking-wide block mb-1.5">
              Download
            </label>
            <button
              onClick={downloadJson}
              className="px-3 py-1.5 text-xs rounded border border-border bg-bg-raised text-text-secondary hover:text-text-primary hover:border-text-muted transition-colors cursor-pointer"
            >
              {investigation.model_id.replace("/", "--")}.json
            </button>
            <p className="text-[10px] text-text-muted mt-1.5">
              Full investigation as JSON. Compatible with the yurai CLI and
              SARIF tooling.
            </p>
          </div>
        </div>
      </div>
    </div>
  );
}
