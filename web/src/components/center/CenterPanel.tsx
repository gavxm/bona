import { useInvestigation } from "../../context/useInvestigation";
import { TabBar } from "./TabBar";
import { DeclaredTab } from "./DeclaredTab";
import { ConfigTab } from "./ConfigTab";
import { CommunityTab } from "./CommunityTab";
import { SourcesTab } from "./SourcesTab";

export function CenterPanel() {
  const { investigation, activeTab } = useInvestigation();
  if (!investigation) return null;

  const sourcesTotal = investigation.sources.length;

  return (
    <div className="h-full flex flex-col bg-bg-surface border-x border-border">
      <div className="px-5 py-3 border-b border-border flex items-center gap-2.5">
        <span className="font-mono text-[11px] font-semibold tracking-widest uppercase text-text-secondary">
          Evidence
        </span>
        <span className="font-mono text-[10.5px] text-text-muted">
          cross-referenced - {sourcesTotal} sources
        </span>
      </div>
      <TabBar />
      <div className="flex-1 min-h-0 overflow-y-auto scroll">
        <div className="px-5 py-1">
          {activeTab === "declared" && <DeclaredTab />}
          {activeTab === "config" && <ConfigTab />}
          {activeTab === "community" && <CommunityTab />}
          {activeTab === "sources" && <SourcesTab />}
        </div>
      </div>
    </div>
  );
}
