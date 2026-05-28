import { Handle, Position, type NodeProps } from "@xyflow/react";
import clsx from "clsx";

export interface ModelNodeData {
  modelId: string;
  license?: string | null;
  isSubject?: boolean;
  exists?: boolean | null;
  highlighted?: boolean;
}

export function ModelNode({ data }: NodeProps) {
  const d = data as unknown as ModelNodeData;
  const shortId = d.modelId.split("/").pop() ?? d.modelId;

  return (
    <div
      className={clsx(
        "px-2.5 py-1.5 rounded border text-center transition-all duration-150 cursor-pointer",
        d.isSubject
          ? "border-accent bg-bg-raised shadow-[0_0_8px_rgba(180,160,230,0.3)]"
          : "border-border bg-bg-surface hover:border-text-muted",
        d.exists === false && "border-dashed border-severity-high/50",
        d.highlighted && "border-accent shadow-[0_0_12px_rgba(180,160,230,0.5)]"
      )}
    >
      <div className="text-[11px] font-mono text-text-primary truncate max-w-35">
        {shortId}
      </div>
      <div className="text-[9px] text-text-muted truncate max-w-35">{d.modelId}</div>
      {d.license && (
        <span className="inline-block mt-0.5 px-1 py-0 text-[9px] rounded bg-bg-base text-text-secondary border border-border">
          {d.license}
        </span>
      )}
      {d.exists === false && (
        <div className="text-[9px] text-severity-high mt-0.5">not found</div>
      )}
      <Handle type="target" position={Position.Top} className="bg-text-muted! w-1.5! h-1.5! border-0!" />
      <Handle type="source" position={Position.Bottom} className="bg-text-muted! w-1.5! h-1.5! border-0!" />
    </div>
  );
}
