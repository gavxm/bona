import { useEffect, useState } from "react";
import { useInvestigation } from "../../context/useInvestigation";
import { FieldRow } from "./FieldRow";
import { useFieldFlags } from "./useFieldFlags";
import { FINDING_LINKS } from "../../linking";

function formatBytes(bytes: number): string {
  const gb = bytes / 1_000_000_000;
  return `${gb.toFixed(1)} GB`;
}

export function ConfigTab() {
  const { investigation } = useInvestigation();
  const config = investigation?.config;
  const flags = useFieldFlags(investigation?.findings ?? []);

  if (!config) {
    return <p className="px-4 py-6 text-sm text-text-muted">Model config not available.</p>;
  }

  return (
    <div className="py-2">
      <FieldRow label="architecture" field="architectures" value={config.architectures.join(", ") || null} mono flag={flags.get("architectures")} />
      <FieldRow label="model type" field="model_type" value={config.model_type} mono flag={flags.get("model_type")} />
      <FieldRow label="hidden size" field="hidden_size" value={config.hidden_size?.toLocaleString() ?? null} mono />
      <FieldRow label="layers" field="num_hidden_layers" value={config.num_hidden_layers?.toString() ?? null} mono />
      <FieldRow label="vocab size" field="vocab_size" value={config.vocab_size?.toLocaleString() ?? null} mono />
      <FieldRow label="weight size" field="safetensors_total_size" value={config.safetensors_total_size != null ? formatBytes(config.safetensors_total_size) : null} mono flag={flags.get("safetensors_total_size")} />
      <FieldRow label="tokenizer" field="tokenizer_class" value={config.tokenizer_class} mono />
      <FieldRow label="quant method" field="quant_method" value={config.quant_method ?? null} mono flag={flags.get("quant_method")} />
      <FieldRow label="quant bits" field="quant_bits" value={config.quant_bits?.toString() ?? null} mono />

      {investigation && <RawConfigBlock modelId={investigation.model_id} />}
    </div>
  );
}

/** Fetches and displays raw config.json with highlighted lines for flagged fields. */
function RawConfigBlock({ modelId }: { modelId: string }) {
  const { investigation } = useInvestigation();
  const [raw, setRaw] = useState<string | null>(null);
  const [error, setError] = useState(false);

  useEffect(() => {
    setRaw(null);
    setError(false);
    fetch(`https://huggingface.co/${modelId}/raw/main/config.json`)
      .then((r) => {
        if (!r.ok) throw new Error();
        return r.text();
      })
      .then(setRaw)
      .catch(() => setError(true));
  }, [modelId]);

  if (error || raw === null) return null;

  // Compute which config keys have findings pointing at them.
  const flaggedKeys = new Set<string>();
  if (investigation) {
    for (const f of investigation.findings) {
      const link = FINDING_LINKS[f.id];
      if (link?.centerTab === "config") {
        for (const field of link.centerFields) flaggedKeys.add(field);
      }
    }
  }

  // Parse into lines and determine highlighting.
  const lines = raw.split("\n");

  return (
    <>
      <div className="font-mono text-[10px] tracking-widest uppercase text-text-muted mt-5 mb-2 flex items-center gap-2.5">
        config.json
        <span className="flex-1 h-px bg-border" />
      </div>
      <pre className="font-mono text-[12.5px] leading-[1.7] bg-bg-base border border-border rounded-lg px-4 py-3.5 text-text-secondary whitespace-pre overflow-x-auto">
        {lines.map((line, i) => {
          // Check if any flagged key appears in this line.
          const isWarn = [...flaggedKeys].some((key) => line.includes(`"${key}"`));
          return (
            <span
              key={i}
              className={isWarn ? "block -mx-4 px-4 bg-severity-medium-bg" : undefined}
            >
              {line}
              {i < lines.length - 1 ? "\n" : ""}
            </span>
          );
        })}
      </pre>
    </>
  );
}
