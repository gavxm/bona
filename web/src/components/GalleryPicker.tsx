import { useEffect, useState } from "react";
import { useInvestigation } from "../context/useInvestigation";

interface GalleryModel {
  id: string;
  file: string;
  tag: string;
  findingCount: number;
}

export function GalleryPicker() {
  const { investigation, loadInvestigation } = useInvestigation();
  const [models, setModels] = useState<GalleryModel[]>([]);

  useEffect(() => {
    fetch(`${import.meta.env.BASE_URL}investigations/gallery.json`)
      .then((r) => r.json())
      .then(setModels)
      .catch(() => {});
  }, []);

  const label = (m: GalleryModel) => {
    const short = m.id.split("/").pop() ?? m.id;
    return m.findingCount > 0
      ? `${short} - ${m.findingCount} finding${m.findingCount !== 1 ? "s" : ""}`
      : `${short} - clean`;
  };

  return (
    <select
      value={investigation?.model_id ?? ""}
      onChange={(e) => {
        if (e.target.value) loadInvestigation(e.target.value);
      }}
      className="bg-bg-raised border border-border rounded px-2 py-1 text-xs text-text-primary cursor-pointer focus:outline-none focus:border-accent"
    >
      <option value="" disabled>
        select a model...
      </option>
      {models.map((m) => (
        <option key={m.id} value={m.id}>
          {label(m)}
        </option>
      ))}
    </select>
  );
}
