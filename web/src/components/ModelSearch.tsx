import { useEffect, useState, useRef } from "react";
import { useInvestigation } from "../context/useInvestigation";

interface GalleryModel {
  id: string;
  file: string;
  tag: string;
  findingCount: number;
}

export function ModelSearch() {
  const { investigation, loading, loadInvestigation } = useInvestigation();
  const [value, setValue] = useState("");
  const [gallery, setGallery] = useState<GalleryModel[]>([]);
  const [open, setOpen] = useState(false);
  const ref = useRef<HTMLDivElement>(null);

  useEffect(() => {
    fetch(`${import.meta.env.BASE_URL}investigations/gallery.json`)
      .then((r) => r.json())
      .then(setGallery)
      .catch(() => {});
  }, []);

  useEffect(() => {
    if (investigation) setValue(investigation.model_id);
  }, [investigation]);

  useEffect(() => {
    function handleClick(e: MouseEvent) {
      if (ref.current && !ref.current.contains(e.target as HTMLElement)) {
        setOpen(false);
      }
    }
    document.addEventListener("mousedown", handleClick);
    return () => document.removeEventListener("mousedown", handleClick);
  }, []);

  const submit = () => {
    const id = value.trim();
    if (id && id.includes("/")) {
      loadInvestigation(id);
      setOpen(false);
    }
  };

  const pick = (id: string) => {
    setValue(id);
    loadInvestigation(id);
    setOpen(false);
  };

  const filtered = value.trim()
    ? gallery.filter((m) => m.id.toLowerCase().includes(value.toLowerCase()))
    : gallery;

  return (
    <div ref={ref} className="relative">
      <div className="flex items-center gap-1">
        <input
          type="text"
          value={value}
          onChange={(e) => {
            setValue(e.target.value);
            setOpen(true);
          }}
          onFocus={() => setOpen(true)}
          onKeyDown={(e) => {
            if (e.key === "Enter") submit();
            if (e.key === "Escape") setOpen(false);
          }}
          placeholder="org/model"
          disabled={loading}
          className="w-56 bg-bg-raised border border-border rounded px-2 py-1 text-xs text-text-primary placeholder:text-text-muted focus:outline-none focus:border-accent disabled:opacity-50"
        />
        <button
          onClick={submit}
          disabled={loading || !value.includes("/")}
          className="px-2 py-1 text-xs rounded border border-border bg-bg-raised text-text-secondary hover:text-text-primary hover:border-text-muted transition-colors disabled:opacity-30 disabled:cursor-not-allowed cursor-pointer"
        >
          {loading ? "..." : "go"}
        </button>
      </div>
      {open && filtered.length > 0 && (
        <div className="absolute right-0 top-full mt-1 w-72 bg-bg-raised border border-border rounded shadow-lg z-50 max-h-60 overflow-y-auto">
          <div className="px-2 py-1 text-[10px] text-text-muted uppercase tracking-wide border-b border-border">
            examples
          </div>
          {filtered.map((m) => (
            <button
              key={m.id}
              onClick={() => pick(m.id)}
              className="w-full text-left px-2 py-1.5 text-xs text-text-secondary hover:bg-bg-surface hover:text-text-primary transition-colors cursor-pointer flex justify-between items-center"
            >
              <span className="font-mono truncate">{m.id}</span>
              <span className="text-[10px] text-text-muted ml-2 shrink-0">
                {m.findingCount > 0
                  ? `${m.findingCount} finding${m.findingCount !== 1 ? "s" : ""}`
                  : "clean"}
              </span>
            </button>
          ))}
        </div>
      )}
    </div>
  );
}
