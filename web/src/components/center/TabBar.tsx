import { useInvestigation } from "../../context/useInvestigation";
import { FINDING_LINKS, type CenterTab } from "../../linking";

const TABS: { id: CenterTab; label: string }[] = [
  { id: "declared", label: "Declared" },
  { id: "config", label: "Config" },
  { id: "community", label: "Community" },
  { id: "sources", label: "Sources" },
];

export function TabBar() {
  const { activeTab, setActiveTab, investigation } = useInvestigation();

  // Count flagged fields per tab.
  const tabFlags: Record<CenterTab, number> = { declared: 0, config: 0, community: 0, sources: 0 };
  if (investigation) {
    for (const f of investigation.findings) {
      const link = FINDING_LINKS[f.id];
      if (link) tabFlags[link.centerTab] += link.centerFields.length;
    }
  }

  return (
    <div className="flex gap-0.5 px-5 border-b border-border shrink-0">
      {TABS.map((tab) => {
        const isActive = activeTab === tab.id;
        const flags = tabFlags[tab.id];
        return (
          <button
            key={tab.id}
            onClick={() => setActiveTab(tab.id)}
            className={[
              "relative px-3.5 py-3 text-[13px] font-medium cursor-pointer flex items-center gap-1.5 transition-colors",
              isActive ? "text-accent-text" : "text-text-muted hover:text-text-secondary",
            ].join(" ")}
          >
            {tab.label}
            {flags > 0 && (
              <span className="font-mono text-[9px] min-w-3.75 h-3.75 px-1 rounded-full inline-flex items-center justify-center bg-severity-high-bg text-severity-high">
                {flags}
              </span>
            )}
            {isActive && (
              <span className="absolute left-2 right-2 -bottom-px h-0.5 bg-accent rounded-full" />
            )}
          </button>
        );
      })}
    </div>
  );
}
