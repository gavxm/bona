import { useInvestigation } from "../../context/useInvestigation";
import { TabBar } from "./TabBar";
import { DeclaredTab } from "./DeclaredTab";
import { ConfigTab } from "./ConfigTab";
import { CommunityTab } from "./CommunityTab";
import { SourcesTab } from "./SourcesTab";

export function CenterPanel() {
  const { investigation, activeTab } = useInvestigation();
  if (!investigation) return null;

  return (
    <div className="h-full overflow-y-auto bg-bg-base border-x border-border">
      <div className="px-4 py-2.5 border-b border-border">
        <h2 className="text-sm font-semibold text-text-primary font-mono">
          {investigation.model_id}
        </h2>
        {investigation.declared.pipeline_tag && (
          <span className="text-[11px] text-text-muted">
            {investigation.declared.pipeline_tag}
          </span>
        )}
      </div>
      <TabBar />
      {activeTab === "declared" && <DeclaredTab />}
      {activeTab === "config" && <ConfigTab />}
      {activeTab === "community" && <CommunityTab />}
      {activeTab === "sources" && <SourcesTab />}
    </div>
  );
}
