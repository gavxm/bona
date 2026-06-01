import { useEffect, useState, useRef, useCallback } from "react";
import { useInvestigation } from "../context/useInvestigation";

interface GalleryModel {
  id: string;
  file: string;
  tag: string;
  findingCount: number;
}

interface HfModel {
  id: string;
  downloads: number;
  likes: number;
}

function formatCount(n: number): string {
  if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(1)}M`;
  if (n >= 1_000) return `${(n / 1_000).toFixed(1)}k`;
  return n.toString();
}

export function ModelSearch() {
  const { investigation, loading, loadInvestigation } = useInvestigation();
  const [draft, setDraft] = useState(
    () => new URLSearchParams(window.location.search).get("model") ?? ""
  );
  const [editing, setEditing] = useState(false);
  const [gallery, setGallery] = useState<GalleryModel[]>([]);
  const [hfResults, setHfResults] = useState<HfModel[]>([]);
  const [open, setOpen] = useState(false);
  const ref = useRef<HTMLDivElement>(null);
  const inputRef = useRef<HTMLInputElement>(null);
  const debounceRef = useRef<ReturnType<typeof setTimeout>>(undefined);

  const value = editing ? draft : (investigation?.model_id ?? draft);

  // Cmd+K / Ctrl+K to focus search.
  useEffect(() => {
    function handleKey(e: KeyboardEvent) {
      if ((e.metaKey || e.ctrlKey) && e.key === "k") {
        e.preventDefault();
        inputRef.current?.focus();
      }
    }
    document.addEventListener("keydown", handleKey);
    return () => document.removeEventListener("keydown", handleKey);
  }, []);

  useEffect(() => {
    fetch(`${import.meta.env.BASE_URL}investigations/gallery.json`)
      .then((r) => r.json())
      .then(setGallery)
      .catch((e) => console.warn("failed to load gallery:", e));
  }, []);

  // Debounced HF search.
  const searchHf = useCallback((query: string) => {
    clearTimeout(debounceRef.current);
    if (!query.trim() || query.trim().length < 2) {
      setHfResults([]);
      return;
    }
    debounceRef.current = setTimeout(async () => {
      try {
        const resp = await fetch(
          `https://huggingface.co/api/models?search=${encodeURIComponent(query.trim())}&sort=downloads&direction=-1&limit=5`
        );
        if (!resp.ok) return;
        const data: HfModel[] = await resp.json();
        setHfResults(data.filter((m) => m.id));
      } catch {
        // Silently fail - HF search is best-effort.
      }
    }, 300);
  }, []);

  useEffect(() => {
    return () => clearTimeout(debounceRef.current);
  }, []);

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
      setEditing(false);
      loadInvestigation(id);
      setOpen(false);
    }
  };

  const pick = (id: string) => {
    setDraft(id);
    setEditing(false);
    loadInvestigation(id);
    setOpen(false);
  };

  const filteredGallery = value.trim()
    ? gallery.filter((m) => m.id.toLowerCase().includes(value.toLowerCase()))
    : gallery;

  // Exclude gallery models from HF results to avoid duplicates.
  const galleryIds = new Set(gallery.map((m) => m.id));
  const filteredHf = hfResults.filter((m) => !galleryIds.has(m.id));

  const hasResults = filteredGallery.length > 0 || filteredHf.length > 0;

  return (
    <div ref={ref} className="relative">
      <div className="flex items-center gap-1">
        <input
          ref={inputRef}
          type="text"
          value={value}
          onChange={(e) => {
            setDraft(e.target.value);
            setEditing(true);
            setOpen(true);
            searchHf(e.target.value);
          }}
          onFocus={() => {
            setDraft(value);
            setEditing(true);
            setOpen(true);
          }}
          onBlur={() => setEditing(false)}
          onKeyDown={(e) => {
            if (e.key === "Enter") submit();
            if (e.key === "Escape") setOpen(false);
          }}
          placeholder="org/model (⌘K)"
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
      {open && hasResults && (
        <div className="absolute right-0 top-full mt-1 w-80 bg-bg-raised border border-border rounded shadow-lg z-50 max-h-72 overflow-y-auto">
          {filteredGallery.length > 0 && (
            <>
              <div className="px-2 py-1 text-[10px] text-text-muted uppercase tracking-wide border-b border-border">
                examples
              </div>
              {filteredGallery.map((m) => (
                <button
                  key={m.id}
                  onMouseDown={() => pick(m.id)}
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
            </>
          )}
          {filteredHf.length > 0 && (
            <>
              <div className="px-2 py-1 text-[10px] text-text-muted uppercase tracking-wide border-b border-border">
                huggingface
              </div>
              {filteredHf.map((m) => (
                <button
                  key={m.id}
                  onMouseDown={() => pick(m.id)}
                  className="w-full text-left px-2 py-1.5 text-xs text-text-secondary hover:bg-bg-surface hover:text-text-primary transition-colors cursor-pointer flex justify-between items-center"
                >
                  <span className="font-mono truncate">{m.id}</span>
                  <span className="text-[10px] text-text-muted ml-2 shrink-0">
                    {formatCount(m.downloads)} dls
                  </span>
                </button>
              ))}
            </>
          )}
        </div>
      )}
    </div>
  );
}
