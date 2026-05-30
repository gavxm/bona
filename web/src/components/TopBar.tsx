import { useInvestigation } from "../context/useInvestigation";
import { ModelSearch } from "./ModelSearch";

function formatTimestamp(iso: string): string {
  try {
    const dt = new Date(iso);
    return dt.toLocaleString("en", {
      month: "short",
      day: "numeric",
      year: "numeric",
      hour: "2-digit",
      minute: "2-digit",
    });
  } catch {
    return iso;
  }
}

function NavButtons() {
  const { canGoBack, canGoForward, goBack, goForward } = useInvestigation();

  if (!canGoBack && !canGoForward) return null;

  const btn = "w-6 h-6 flex items-center justify-center rounded border border-border transition-colors cursor-pointer";
  const active = "text-text-secondary hover:text-text-primary hover:border-text-muted";
  const disabled = "text-text-muted/30 border-border/50 cursor-default";

  return (
    <div className="flex items-center gap-0.5">
      <button onClick={goBack} disabled={!canGoBack} className={`${btn} ${canGoBack ? active : disabled}`}>
        <svg width="10" height="10" viewBox="0 0 10 10" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round">
          <path d="M6.5 1.5L3 5l3.5 3.5" />
        </svg>
      </button>
      <button onClick={goForward} disabled={!canGoForward} className={`${btn} ${canGoForward ? active : disabled}`}>
        <svg width="10" height="10" viewBox="0 0 10 10" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round">
          <path d="M3.5 1.5L7 5l-3.5 3.5" />
        </svg>
      </button>
    </div>
  );
}

export function TopBar() {
  const { investigation, isSnapshot } = useInvestigation();

  return (
    <div className="h-12 flex items-center justify-between px-4 border-b border-border bg-bg-base">
      <div className="flex items-center gap-5">
        <div className="flex items-center gap-1.5">
          <img src={`${import.meta.env.BASE_URL}logo-dark.svg`} alt="yurai" className="h-8 w-auto" />
          <span className="text-text-primary font-semibold text-base tracking-tight">yurai</span>
          <span className="text-text-muted text-[12px] ml-2">provenance explorer</span>
        </div>
        <NavButtons />
        {investigation && (
          <>
            <span className="text-border text-xs">|</span>
            <span className="text-text-secondary text-xs font-mono">
              {investigation.model_id}
            </span>
            {isSnapshot && (
              <span className="text-[10px] px-1.5 py-0.5 rounded bg-bg-raised border border-border text-accent">
                snapshot
              </span>
            )}
            <span className="text-text-muted text-[10px]">
              investigated {formatTimestamp(investigation.investigated_at)}
            </span>
          </>
        )}
      </div>
      <ModelSearch />
    </div>
  );
}
