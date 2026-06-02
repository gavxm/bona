import { TopBar } from "./TopBar";
import { VerdictHeader } from "./VerdictHeader";
import { LineageGraph } from "./left/LineageGraph";
import { CenterPanel } from "./center/CenterPanel";
import { RightPanel } from "./right/RightPanel";
import { useInvestigation } from "../context/useInvestigation";
import { LogoMark } from "./shared/LogoMark";

function EmptyState() {
  const { loadInvestigation } = useInvestigation();

  const examples = [
    { id: "ruslanmv/Medical-Llama3-8B", label: "license violation" },
    { id: "microsoft/phi-2", label: "missing docs" },
    { id: "google/gemma-2b", label: "clean report" },
  ];

  return (
    <div className="flex-1 flex items-center justify-center">
      <div className="text-center max-w-md">
        <div className="mb-4">
          <LogoMark />
        </div>
        <p className="text-text-primary text-base mb-1">
          Provenance explorer for AI models
        </p>
        <p className="text-text-secondary text-sm mb-6">
          Investigate HuggingFace models for license violations, lineage
          inconsistencies, and trust signals. Type any <span className="font-mono text-text-primary">org/model</span> above or try
          an example:
        </p>
        <div className="flex flex-col gap-1.5 items-center">
          {examples.map((ex) => (
            <button
              key={ex.id}
              onClick={() => loadInvestigation(ex.id)}
              className="text-sm text-link hover:underline cursor-pointer flex items-center gap-2"
            >
              <span className="font-mono">{ex.id}</span>
              <span className="text-text-muted text-xs">- {ex.label}</span>
            </button>
          ))}
        </div>
      </div>
    </div>
  );
}

function LoadingIndicator() {
  return (
    <div className="flex-1 flex items-center justify-center">
      <div className="flex flex-col items-center gap-4">
        <LogoMark animate />
        <span className="text-text-muted text-xs">investigating</span>
      </div>
    </div>
  );
}

function ErrorState({ error }: { error: string }) {
  const modelId = new URLSearchParams(window.location.search).get("model");

  return (
    <div className="flex-1 flex items-center justify-center">
      <div className="text-center max-w-sm">
        <p className="text-severity-high text-sm mb-3">{error}</p>
        {modelId && (
          <a
            href={`https://huggingface.co/${modelId}`}
            target="_blank"
            rel="noopener noreferrer"
            className="text-xs text-link hover:underline"
          >
            View {modelId} on HuggingFace
          </a>
        )}
      </div>
    </div>
  );
}

/** Narrow read-only view: verdict + findings stacked vertically. */
function MobileView() {
  const { investigation } = useInvestigation();
  if (!investigation) return null;

  return (
    <div className="overflow-y-auto scroll">
      <VerdictHeader />
      <RightPanel />
    </div>
  );
}

export function AppShell() {
  const { investigation, loading, error } = useInvestigation();

  return (
    <div className="h-screen flex flex-col">
      <TopBar />
      {loading && <LoadingIndicator />}
      {error && <ErrorState error={error} />}
      {!loading && !error && !investigation && <EmptyState />}
      {!loading && !error && investigation && (
        <>
          {/* Desktop: 3-panel layout */}
          <div className="hidden lg:flex flex-col flex-1 min-h-0">
            <VerdictHeader />
            <div className="flex-1 grid grid-cols-[300px_1fr_380px] min-h-0">
              <LineageGraph />
              <CenterPanel />
              <RightPanel />
            </div>
          </div>
          {/* Mobile: stacked verdict + findings */}
          <div className="lg:hidden flex-1 min-h-0">
            <MobileView />
          </div>
        </>
      )}
    </div>
  );
}
