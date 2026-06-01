import { useState } from "react";
import { useInvestigation } from "../../context/useInvestigation";
import { FieldRow, useFieldFlags } from "./FieldRow";

function formatNumber(n: number): string {
  return n.toLocaleString();
}

function formatDate(iso: string): string {
  try {
    const dt = new Date(iso);
    return dt.toLocaleString("en", { month: "short", day: "numeric", year: "numeric" });
  } catch {
    return iso;
  }
}

const MAX_VISIBLE_TAGS = 8;

const DANGEROUS_EXTENSIONS = [".pkl", ".pth", ".exe", ".bat", ".cmd", ".msi", ".scr"];
const PICKLE_EXTENSIONS = [".pkl", ".pth"];

function isDangerous(filename: string): boolean {
  return DANGEROUS_EXTENSIONS.some((ext) => filename.endsWith(ext));
}

function dangerTag(filename: string): string | null {
  if (PICKLE_EXTENSIONS.some((ext) => filename.endsWith(ext))) return "pickle";
  if ([".exe", ".bat", ".cmd", ".msi", ".scr"].some((ext) => filename.endsWith(ext))) return "executable";
  return null;
}

export function DeclaredTab() {
  const { investigation, highlightedFields } = useInvestigation();
  const [showAllTags, setShowAllTags] = useState(false);
  if (!investigation) return null;

  const d = investigation.declared;
  const flags = useFieldFlags(investigation.findings);
  const visibleTags = showAllTags ? d.tags : d.tags.slice(0, MAX_VISIBLE_TAGS);
  const hiddenCount = d.tags.length - MAX_VISIBLE_TAGS;
  const filesHighlighted = highlightedFields.includes("files");

  return (
    <div className="py-2">
      <FieldRow label="license" field="declared_license" value={d.declared_license} flag={flags.get("declared_license")} />
      <FieldRow label="base model" field="declared_base_model" value={d.declared_base_model} mono flag={flags.get("declared_base_model")} />
      <FieldRow label="library" field="library" value={d.library} />
      <FieldRow label="pipeline" field="pipeline_tag" value={d.pipeline_tag} />
      <FieldRow
        label="downloads"
        field="downloads"
        value={d.downloads != null ? formatNumber(d.downloads) : null}
        mono
        flag={flags.get("downloads")}
      />
      <FieldRow
        label="likes"
        field="likes"
        value={d.likes != null ? formatNumber(d.likes) : null}
        mono
        flag={flags.get("likes")}
      />
      <FieldRow label="gated" field="gated" value={d.gated ?? null} flag={flags.get("gated")} />
      <FieldRow label="created" field="created_at" value={d.created_at ? formatDate(d.created_at) : null} flag={flags.get("created_at")} />
      <FieldRow label="last modified" field="last_modified" value={d.last_modified ? formatDate(d.last_modified) : null} flag={flags.get("last_modified")} />

      {/* Files list */}
      {d.files && d.files.length > 0 && (
        <>
          <div className="font-mono text-[10px] tracking-widest uppercase text-text-muted mt-5 mb-2 flex items-center gap-2.5">
            files <span className="font-sans text-[11px] tracking-normal normal-case">{d.files.length} of {d.files.length}</span>
            <span className="flex-1 h-px bg-border" />
          </div>
          <div
            className={[
              "rounded-lg -mx-2.5 px-2.5 transition-all duration-300",
              filesHighlighted ? "bg-accent-bg shadow-[inset_0_0_0_1px_var(--color-accent-line)]" : "",
            ].join(" ")}
          >
            {d.files.map((fn) => {
              const danger = isDangerous(fn);
              const tag = dangerTag(fn);
              return (
                <div key={fn} className="flex items-center gap-2.5 py-1.5 border-t border-border first:border-t-0 font-mono text-[12px]">
                  <span className={`w-1.5 h-1.5 rounded-full shrink-0 ${danger ? "bg-severity-high" : "bg-text-muted"}`} />
                  <span className={`flex-1 min-w-0 truncate ${danger ? "text-severity-high" : "text-text-primary"}`}>
                    {fn}
                  </span>
                  {tag && (
                    <span className="text-[9px] tracking-wide px-1.5 py-0.5 rounded border bg-severity-high-bg text-severity-high border-severity-high-line shrink-0">
                      {tag}
                    </span>
                  )}
                </div>
              );
            })}
          </div>
        </>
      )}

      {/* Tags */}
      {d.tags.length > 0 && (
        <>
          <div className="font-mono text-[10px] tracking-widest uppercase text-text-muted mt-5 mb-2 flex items-center gap-2.5">
            tags
            <span className="flex-1 h-px bg-border" />
          </div>
          <div className="flex flex-wrap gap-1.5">
            {visibleTags.map((tag) => (
              <span
                key={tag}
                className="font-mono text-[11px] text-text-secondary bg-bg-raised border border-border px-2 py-0.5 rounded-md"
              >
                {tag}
              </span>
            ))}
            {!showAllTags && hiddenCount > 0 && (
              <button
                onClick={() => setShowAllTags(true)}
                className="text-[11px] text-accent-text hover:underline cursor-pointer px-2 py-0.5"
              >
                +{hiddenCount} more
              </button>
            )}
          </div>
        </>
      )}
    </div>
  );
}
