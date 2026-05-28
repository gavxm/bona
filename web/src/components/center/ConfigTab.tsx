import { useInvestigation } from "../../context/useInvestigation";
import { FieldRow } from "./FieldRow";

function formatBytes(bytes: number): string {
  const gb = bytes / 1_000_000_000;
  return `${gb.toFixed(1)} GB`;
}

export function ConfigTab() {
  const { investigation } = useInvestigation();
  const config = investigation?.config;
  if (!config) {
    return <p className="px-4 py-6 text-sm text-text-muted">No config data available.</p>;
  }

  return (
    <div className="py-2">
      <FieldRow
        label="architecture"
        field="architectures"
        value={config.architectures.join(", ") || null}
        mono
      />
      <FieldRow label="model type" field="model_type" value={config.model_type} mono />
      <FieldRow
        label="hidden size"
        field="hidden_size"
        value={config.hidden_size?.toLocaleString() ?? null}
        mono
      />
      <FieldRow
        label="layers"
        field="num_hidden_layers"
        value={config.num_hidden_layers?.toString() ?? null}
        mono
      />
      <FieldRow
        label="vocab size"
        field="vocab_size"
        value={config.vocab_size?.toLocaleString() ?? null}
        mono
      />
      <FieldRow
        label="weight size"
        field="safetensors_total_size"
        value={config.safetensors_total_size != null ? formatBytes(config.safetensors_total_size) : null}
        mono
      />
      <FieldRow label="tokenizer" field="tokenizer_class" value={config.tokenizer_class} mono />
    </div>
  );
}
