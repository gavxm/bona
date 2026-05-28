import clsx from "clsx";
import { useInvestigation } from "../../context/useInvestigation";
import type { CenterTab } from "../../linking";

const TABS: { id: CenterTab; label: string }[] = [
  { id: "declared", label: "Declared" },
  { id: "config", label: "Config" },
  { id: "community", label: "Community" },
  { id: "sources", label: "Sources" },
];

export function TabBar() {
  const { activeTab, setActiveTab } = useInvestigation();

  return (
    <div className="flex border-b border-border">
      {TABS.map((tab) => (
        <button
          key={tab.id}
          onClick={() => setActiveTab(tab.id)}
          className={clsx(
            "px-4 py-2 text-xs font-medium transition-colors cursor-pointer",
            activeTab === tab.id
              ? "text-accent border-b-2 border-b-accent bg-bg-raised"
              : "text-text-secondary hover:text-text-primary"
          )}
        >
          {tab.label}
        </button>
      ))}
    </div>
  );
}
