import { TopBar } from "./TopBar";
import { LineageGraph } from "./left/LineageGraph";
import { CenterPanel } from "./center/CenterPanel";
import { RightPanel } from "./right/RightPanel";
import { useInvestigation } from "../context/useInvestigation";

export function AppShell() {
  const { investigation, loading, error } = useInvestigation();

  return (
    <div className="h-screen flex flex-col">
      <TopBar />
      {loading && (
        <div className="flex-1 flex items-center justify-center text-text-muted">
          investigating...
        </div>
      )}
      {error && (
        <div className="flex-1 flex items-center justify-center text-severity-high">
          {error}
        </div>
      )}
      {!loading && !error && !investigation && (
        <div className="flex-1 flex items-center justify-center text-text-muted">
          <div className="text-center">
            <p className="text-accent font-bold text-lg mb-2">◁ bona ▷</p>
            <p className="text-text-secondary text-sm">select a model to investigate</p>
          </div>
        </div>
      )}
      {!loading && !error && investigation && (
        <div className="flex-1 grid grid-cols-[280px_1fr_340px] min-h-0">
          <LineageGraph />
          <CenterPanel />
          <RightPanel />
        </div>
      )}
    </div>
  );
}
