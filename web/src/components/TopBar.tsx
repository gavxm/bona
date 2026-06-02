import { useState } from "react";
import { useInvestigation } from "../context/useInvestigation";
import { ModelSearch } from "./ModelSearch";
import { ExportModal } from "./ExportModal";
import { IconChevronLeft, IconChevronRight, IconDownload } from "./shared/Icons";

function NavButtons() {
  const { canGoBack, canGoForward, goBack, goForward } = useInvestigation();

  if (!canGoBack && !canGoForward) return null;

  const btn = "w-6 h-6 flex items-center justify-center rounded border border-border transition-colors cursor-pointer";
  const active = "text-text-secondary hover:text-text-primary hover:border-border-strong";
  const disabled = "text-text-muted/30 border-border/50 cursor-default";

  return (
    <div className="flex items-center gap-0.5">
      <button onClick={goBack} disabled={!canGoBack} className={`${btn} ${canGoBack ? active : disabled}`}>
        <IconChevronLeft />
      </button>
      <button onClick={goForward} disabled={!canGoForward} className={`${btn} ${canGoForward ? active : disabled}`}>
        <IconChevronRight />
      </button>
    </div>
  );
}

export function TopBar() {
  const { investigation, isSnapshot } = useInvestigation();
  const [showExport, setShowExport] = useState(false);

  return (
    <>
      <div className="h-13 flex items-center gap-4 px-4.5 border-b border-border bg-bg-surface shrink-0">
        {/* Brand */}
        <a
          href={import.meta.env.BASE_URL}
          className="flex items-center gap-2.5 hover:opacity-80 transition-opacity shrink-0"
        >
          <img src={`${import.meta.env.BASE_URL}logo-dark.svg`} alt="yurai" className="h-8 w-auto" />
          <span className="text-text-primary font-bold text-[17px] tracking-tight">yurai</span>
          <span className="font-mono text-[10px] tracking-widest uppercase text-text-muted hidden sm:inline">provenance explorer</span>
        </a>

        <div className="w-px h-5.5 bg-border hidden lg:block" />

        <span className="hidden lg:flex"><NavButtons /></span>

        {isSnapshot && (
          <span className="font-mono text-[10px] px-1.5 py-0.5 rounded bg-accent-bg border border-accent-line text-accent-text">
            snapshot
          </span>
        )}

        {/* Spacer */}
        <span className="flex-1" />

        {/* Search */}
        <span className="hidden lg:block"><ModelSearch /></span>

        {/* Export */}
        {investigation && (
          <button
            onClick={() => setShowExport(true)}
            className="h-[34px] px-3.5 rounded-lg border border-border-strong bg-bg-raised text-text-secondary text-[12.5px] font-medium flex items-center gap-1.5 cursor-pointer hover:text-text-primary hover:bg-bg-open transition-colors shrink-0"
          >
            <IconDownload size={13} />
            <span className="hidden sm:inline">export report</span>
          </button>
        )}
      </div>
      {showExport && <ExportModal onClose={() => setShowExport(false)} />}
    </>
  );
}
