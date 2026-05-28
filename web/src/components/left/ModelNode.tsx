import { Handle, Position, type NodeProps, type Node } from "@xyflow/react";
import clsx from "clsx";

export type ModelNodeData = {
  modelId: string;
  license?: string | null;
  isSubject?: boolean;
  exists?: boolean | null;
  highlighted?: boolean;
};

export type ModelNodeType = Node<ModelNodeData, "model">;

export function ModelNode({ data }: NodeProps<ModelNodeType>) {
  const shortId = data.modelId.split("/").pop() ?? data.modelId;

  return (
    <div
      className={clsx(
        "rounded border text-center cursor-pointer",
        "transition-all duration-300 ease-in-out",
        data.highlighted
          ? "px-3 py-2 border-accent shadow-[0_0_16px_rgba(180,160,230,0.6)] scale-105 bg-bg-raised"
          : data.isSubject
            ? "px-2.5 py-1.5 border-accent bg-bg-raised shadow-[0_0_8px_rgba(180,160,230,0.3)]"
            : "px-2.5 py-1.5 border-text-muted/60 bg-bg-raised hover:border-text-secondary",
        data.exists === false && "border-dashed border-severity-high/50",
      )}
    >
      <div className={clsx(
        "font-mono text-text-primary truncate max-w-35",
        data.highlighted ? "text-xs font-semibold" : "text-[11px]"
      )}>
        {shortId}
      </div>
      <div className="text-[9px] text-text-secondary truncate max-w-35">{data.modelId}</div>
      {data.license && (
        <span className="inline-block mt-0.5 px-1 py-0 text-[9px] rounded bg-bg-base text-text-primary border border-text-muted/40">
          {data.license}
        </span>
      )}
      {data.exists === false && (
        <div className="text-[9px] text-severity-high mt-0.5">not found</div>
      )}
      <Handle type="target" position={Position.Top} className="bg-text-muted! w-1.5! h-1.5! border-0!" />
      <Handle type="source" position={Position.Bottom} className="bg-text-muted! w-1.5! h-1.5! border-0!" />
    </div>
  );
}
