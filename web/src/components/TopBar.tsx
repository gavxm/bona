import { useInvestigation } from "../context/useInvestigation";
import { GalleryPicker } from "./GalleryPicker";

export function TopBar() {
  const { investigation } = useInvestigation();

  return (
    <div className="h-12 flex items-center justify-between px-4 border-b border-border bg-bg-base">
      <div className="flex items-center gap-4">
        <span className="text-accent font-bold text-sm">◁ bona ▷</span>
        {investigation && (
          <span className="text-text-secondary text-xs font-mono">
            {investigation.model_id}
          </span>
        )}
      </div>
      <GalleryPicker />
    </div>
  );
}
