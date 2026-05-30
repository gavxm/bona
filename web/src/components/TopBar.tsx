import { useInvestigation } from "../context/useInvestigation";
import { GalleryPicker } from "./GalleryPicker";

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

export function TopBar() {
  const { investigation } = useInvestigation();

  return (
    <div className="h-12 flex items-center justify-between px-4 border-b border-border bg-bg-base">
      <div className="flex items-center gap-5">
        <div className="flex items-center gap-1.5">
          <img src={`${import.meta.env.BASE_URL}logo-dark.svg`} alt="yurai" className="h-8 w-auto" />
          <span className="text-text-primary font-semibold text-base tracking-tight">yurai</span>
          <span className="text-text-muted text-[12px] ml-2">provenance explorer</span>
        </div>
        {investigation && (
          <>
            <span className="text-border text-xs">│</span>
            <span className="text-text-secondary text-xs font-mono">
              {investigation.model_id}
            </span>
            <span className="text-text-muted text-[10px]">
              investigated {formatTimestamp(investigation.investigated_at)}
            </span>
          </>
        )}
      </div>
      <GalleryPicker />
    </div>
  );
}
