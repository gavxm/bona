import { useInvestigation } from "../context/useInvestigation";

const GALLERY_MODELS = [
  { id: "ruslanmv/Medical-Llama3-8B", label: "Medical-Llama3-8B", tag: "license violation" },
  { id: "microsoft/phi-2", label: "phi-2", tag: "clean model" },
  { id: "TheBloke/Llama-2-7B-Chat-GGUF", label: "Llama-2-7B-Chat-GGUF", tag: "full lineage" },
];

export function GalleryPicker() {
  const { investigation, loadInvestigation } = useInvestigation();

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
      {GALLERY_MODELS.map((m) => (
        <option key={m.id} value={m.id}>
          {m.label} - {m.tag}
        </option>
      ))}
    </select>
  );
}
