import { useState } from "react";
import { useInvestigation } from "../../context/useInvestigation";
import { FieldRow } from "./FieldRow";

function formatNumber(n: number): string {
  return n.toLocaleString();
}

const MAX_VISIBLE_TAGS = 8;

export function DeclaredTab() {
  const { investigation } = useInvestigation();
  const [showAllTags, setShowAllTags] = useState(false);
  if (!investigation) return null;

  const d = investigation.declared;
  const visibleTags = showAllTags ? d.tags : d.tags.slice(0, MAX_VISIBLE_TAGS);
  const hiddenCount = d.tags.length - MAX_VISIBLE_TAGS;

  return (
    <div className="py-1">
      <FieldRow label="license" field="declared_license" value={d.declared_license} />
      <FieldRow label="base model" field="declared_base_model" value={d.declared_base_model} mono />
      <FieldRow label="library" field="library" value={d.library} />
      <FieldRow label="pipeline" field="pipeline_tag" value={d.pipeline_tag} />
      <FieldRow
        label="downloads"
        field="downloads"
        value={d.downloads != null ? formatNumber(d.downloads) : null}
        mono
      />
      {d.tags.length > 0 && (
        <div className="px-4 py-2 border-l-3 border-l-transparent">
          <span className="text-text-secondary text-xs w-36 inline-block">tags</span>
          <div className="flex flex-wrap gap-1 mt-1">
            {visibleTags.map((tag) => (
              <span
                key={tag}
                className="inline-block px-1.5 py-0.5 text-[11px] rounded bg-bg-raised text-text-muted border border-border"
              >
                {tag}
              </span>
            ))}
            {!showAllTags && hiddenCount > 0 && (
              <button
                onClick={() => setShowAllTags(true)}
                className="inline-block px-1.5 py-0.5 text-[11px] rounded text-accent hover:underline cursor-pointer"
              >
                +{hiddenCount} more
              </button>
            )}
          </div>
        </div>
      )}
    </div>
  );
}
