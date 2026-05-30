import { TopBar } from "./TopBar";
import { SummaryStrip } from "./SummaryStrip";
import { LineageGraph } from "./left/LineageGraph";
import { CenterPanel } from "./center/CenterPanel";
import { RightPanel } from "./right/RightPanel";
import { useInvestigation } from "../context/useInvestigation";

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
        <img
          src={`${import.meta.env.BASE_URL}logo-dark.svg`}
          alt="yurai"
          className="h-20 w-auto mx-auto opacity-60 mb-4"
        />
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

function LoadingSkeleton() {
  return (
    <>
      {/* fake summary strip */}
      <div className="flex items-center gap-5 px-4 py-1.5 border-b border-border bg-bg-surface text-[11px]">
        <div className="h-3 w-40 rounded bg-bg-raised animate-pulse" />
        <div className="h-3 w-20 rounded bg-bg-raised animate-pulse" />
      </div>
      <div className="flex-1 grid grid-cols-[320px_1fr_340px] min-h-0">
        {/* left panel skeleton */}
        <div className="bg-bg-surface border-r border-border p-4">
          <div className="h-3 w-16 rounded bg-bg-raised animate-pulse mb-6" />
          <div className="space-y-16 mt-4">
            <div className="h-14 w-40 mx-auto rounded border border-border bg-bg-raised animate-pulse" />
            <div className="h-14 w-40 mx-auto rounded border border-border bg-bg-raised animate-pulse" />
          </div>
        </div>
        {/* center panel skeleton */}
        <div className="bg-bg-base border-x border-border p-4">
          <div className="h-4 w-48 rounded bg-bg-raised animate-pulse mb-4" />
          <div className="h-3 w-24 rounded bg-bg-raised animate-pulse mb-6" />
          <div className="space-y-3">
            {Array.from({ length: 5 }).map((_, i) => (
              <div key={i} className="flex gap-4">
                <div className="h-3 w-28 rounded bg-bg-raised animate-pulse" />
                <div className="h-3 w-36 rounded bg-bg-raised animate-pulse" />
              </div>
            ))}
          </div>
        </div>
        {/* right panel skeleton */}
        <div className="bg-bg-surface p-4">
          <div className="h-3 w-16 rounded bg-bg-raised animate-pulse mb-4" />
          <div className="space-y-4">
            {Array.from({ length: 2 }).map((_, i) => (
              <div key={i} className="space-y-2">
                <div className="h-5 w-16 rounded bg-bg-raised animate-pulse" />
                <div className="h-3 w-full rounded bg-bg-raised animate-pulse" />
                <div className="h-3 w-3/4 rounded bg-bg-raised animate-pulse" />
              </div>
            ))}
          </div>
        </div>
      </div>
    </>
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

export function AppShell() {
  const { investigation, loading, error } = useInvestigation();

  return (
    <div className="h-screen flex flex-col">
      <TopBar />
      {loading && <LoadingSkeleton />}
      {error && <ErrorState error={error} />}
      {!loading && !error && !investigation && <EmptyState />}
      {!loading && !error && investigation && (
        <>
          <SummaryStrip />
          <div className="flex-1 grid grid-cols-[320px_1fr_340px] min-h-0">
            <LineageGraph />
            <CenterPanel />
            <RightPanel />
          </div>
        </>
      )}
    </div>
  );
}
