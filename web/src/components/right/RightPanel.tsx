import { useInvestigation } from "../../context/useInvestigation";
import { FindingsHeader } from "./FindingsHeader";
import { FindingCard } from "./FindingCard";

export function RightPanel() {
  const { investigation } = useInvestigation();
  if (!investigation) return null;

  const { findings } = investigation;

  return (
    <div className="h-full overflow-y-auto bg-bg-surface">
      <FindingsHeader findings={findings} />
      {findings.length === 0 ? (
        <p className="px-4 py-6 text-xs text-status-ok">No issues found.</p>
      ) : (
        <div className="divide-y divide-border">
          {findings.map((f) => (
            <FindingCard key={f.id} finding={f} />
          ))}
        </div>
      )}
    </div>
  );
}
