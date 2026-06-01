import { TopBar } from "./TopBar";
import { VerdictHeader } from "./VerdictHeader";
import { LineageGraph } from "./left/LineageGraph";
import { CenterPanel } from "./center/CenterPanel";
import { RightPanel } from "./right/RightPanel";
import { useInvestigation } from "../context/useInvestigation";

type IdleStyle = "traverse" | "pulse" | "orbit" | "scan";

function LogoMark({ animate = false, idle = "scan" }: { animate?: boolean; idle?: IdleStyle }) {
  const outlines = (
    <>
      <circle cx="22" cy="32" r="20" stroke="#FFFFFF" strokeWidth="3.6" />
      <circle cx="44" cy="32" r="20" stroke="#FFFFFF" strokeWidth="3.6" />
      <circle cx="66" cy="32" r="20" stroke="#FFFFFF" strokeWidth="3.6" />
      <circle cx="88" cy="32" r="20" stroke="#FFFFFF" strokeWidth="3.6" />
    </>
  );

  // Loading: fast traversal
  if (animate) {
    return (
      <svg width="112" height="64" viewBox="0 0 112 64" fill="none" xmlns="http://www.w3.org/2000/svg" className="mx-auto">
        {outlines}
        <circle cx="22" cy="32" r="20" fill="#FFFFFF">
          <animate attributeName="cx" values="22;44;66;88;88;66;44;22" dur="2s" repeatCount="indefinite"
            keyTimes="0;0.15;0.3;0.45;0.55;0.7;0.85;1" calcMode="spline"
            keySplines="0.4 0 0.2 1;0.4 0 0.2 1;0.4 0 0.2 1;0.4 0 0.2 1;0.4 0 0.2 1;0.4 0 0.2 1;0.4 0 0.2 1" />
        </circle>
      </svg>
    );
  }

  return (
    <svg width="112" height="64" viewBox="0 0 112 64" fill="none" xmlns="http://www.w3.org/2000/svg" className="mx-auto">
      {outlines}

      {/* A: Traverse - filled circle glides left-right slowly */}
      {idle === "traverse" && (
        <circle cx="88" cy="32" r="20" fill="#FFFFFF">
          <animate attributeName="cx" values="22;44;66;88;88;66;44;22" dur="4s" repeatCount="indefinite"
            keyTimes="0;0.15;0.3;0.45;0.55;0.7;0.85;1" calcMode="spline"
            keySplines="0.4 0 0.2 1;0.4 0 0.2 1;0.4 0 0.2 1;0.4 0 0.2 1;0.4 0 0.2 1;0.4 0 0.2 1;0.4 0 0.2 1" />
        </circle>
      )}

      {/* B: Pulse - filled circle breathes at home position */}
      {idle === "pulse" && (
        <circle cx="88" cy="32" r="20" fill="#FFFFFF">
          <animate attributeName="r" values="20;17;20" dur="3s" repeatCount="indefinite" calcMode="spline"
            keySplines="0.4 0 0.6 1;0.4 0 0.6 1" />
          <animate attributeName="opacity" values="1;0.6;1" dur="3s" repeatCount="indefinite" calcMode="spline"
            keySplines="0.4 0 0.6 1;0.4 0 0.6 1" />
        </circle>
      )}

      {/* C: Orbit - filled circle drifts in a lazy ellipse around the logo */}
      {idle === "orbit" && (
        <circle cx="55" cy="32" r="20" fill="#FFFFFF">
          <animate attributeName="cx" values="88;66;22;44;88" dur="6s" repeatCount="indefinite" calcMode="spline"
            keySplines="0.5 0 0.5 1;0.5 0 0.5 1;0.5 0 0.5 1;0.5 0 0.5 1" />
          <animate attributeName="cy" values="32;22;32;42;32" dur="6s" repeatCount="indefinite" calcMode="spline"
            keySplines="0.5 0 0.5 1;0.5 0 0.5 1;0.5 0 0.5 1;0.5 0 0.5 1" />
        </circle>
      )}

      {/* D: Scan - circles fill one by one left-to-right, then empty right-to-left */}
      {idle === "scan" && [22, 44, 66, 88].map((cx, i) => (
        <circle key={cx} cx={cx} cy={32} r="20" fill="#FFFFFF">
          <animate attributeName="opacity" values="0;0;1;1;0;0" dur="4s" repeatCount="indefinite"
            begin={`${i * 0.5}s`} keyTimes="0;0.05;0.15;0.5;0.6;1" calcMode="spline"
            keySplines="0.4 0 0.6 1;0.4 0 0.6 1;0.4 0 0.6 1;0.4 0 0.6 1;0.4 0 0.6 1" />
        </circle>
      ))}
    </svg>
  );
}

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
