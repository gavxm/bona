import { useInvestigation } from "../../context/useInvestigation";
import { SeverityBadge } from "../shared/SeverityBadge";
import { TabBar } from "./TabBar";
import { DeclaredTab } from "./DeclaredTab";
import { ConfigTab } from "./ConfigTab";
import { CommunityTab } from "./CommunityTab";
import { SourcesTab } from "./SourcesTab";

function Chip({ children }: { children: React.ReactNode }) {
  return (
    <span className="inline-block px-1.5 py-0.5 text-[10px] rounded bg-bg-raised text-text-secondary border border-border">
      {children}
    </span>
  );
}

export function CenterPanel() {
  const { investigation, activeTab } = useInvestigation();
  if (!investigation) return null;

  const highCount = investigation.findings.filter((f) => f.severity === "high").length;
  const medCount = investigation.findings.filter((f) => f.severity === "medium").length;
  const totalFindings = investigation.findings.length;

  return (
    <div className="h-full overflow-y-auto bg-bg-base border-x border-border">
      <div className="px-4 py-2 border-b border-border">
        <div className="flex items-center justify-between">
          <div>
            <h2 className="text-sm font-semibold text-text-primary font-mono">
              {investigation.model_id}
            </h2>
            <div className="flex items-center gap-1.5 mt-1">
              {investigation.declared.pipeline_tag && (
                <Chip>{investigation.declared.pipeline_tag}</Chip>
              )}
              {investigation.declared.library && (
                <Chip>{investigation.declared.library}</Chip>
              )}
              {investigation.declared.declared_license && (
                <Chip>{investigation.declared.declared_license}</Chip>
              )}
            </div>
          </div>
          <div className="flex items-center gap-2">
            {totalFindings === 0 ? (
              <span className="text-[11px] text-status-ok">clean</span>
            ) : (
              <>
                {highCount > 0 && <SeverityBadge severity="high" />}
                {medCount > 0 && <SeverityBadge severity="medium" />}
                {highCount === 0 && medCount === 0 && (
                  <span className="text-[11px] text-text-muted">
                    {totalFindings} minor finding{totalFindings !== 1 ? "s" : ""}
                  </span>
                )}
              </>
            )}
          </div>
        </div>
      </div>
      <TabBar />
      {activeTab === "declared" && <DeclaredTab />}
      {activeTab === "config" && <ConfigTab />}
      {activeTab === "community" && <CommunityTab />}
      {activeTab === "sources" && <SourcesTab />}
    </div>
  );
}
