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
        "rounded border text-center cursor-pointer",
        "transition-all duration-300 ease-in-out",
        d.highlighted
          ? "px-3 py-2 border-accent shadow-[0_0_16px_rgba(180,160,230,0.6)] scale-105 bg-bg-raised"
          : d.isSubject
            ? "px-2.5 py-1.5 border-accent bg-bg-raised shadow-[0_0_8px_rgba(180,160,230,0.3)]"
            : "px-2.5 py-1.5 border-text-muted/60 bg-bg-raised hover:border-text-secondary",
        d.exists === false && "border-dashed border-severity-high/50",
      )}
    >
      <div className={clsx(
        "font-mono text-text-primary truncate max-w-35",
        d.highlighted ? "text-xs font-semibold" : "text-[11px]"
      )}>
        {shortId}
      </div>
      <div className="text-[9px] text-text-secondary truncate max-w-35">{d.modelId}</div>
      {d.license && (
        <span className="inline-block mt-0.5 px-1 py-0 text-[9px] rounded bg-bg-base text-text-primary border border-text-muted/40">
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
