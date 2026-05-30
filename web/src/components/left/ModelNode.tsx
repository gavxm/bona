import { Handle, Position, type NodeProps, type Node } from "@xyflow/react";
import clsx from "clsx";

export type ModelNodeData = {
  modelId: string;
  license?: string | null;
  isSubject?: boolean;
  exists?: boolean | null;
  highlighted?: boolean;
  hasHighFinding?: boolean;
  hasMediumFinding?: boolean;
};

export type ModelNodeType = Node<ModelNodeData, "model">;

export function ModelNode({ data }: NodeProps<ModelNodeType>) {
  const shortId = data.modelId.split("/").pop() ?? data.modelId;

  // Priority: highlighted glow > finding glow > subject glow > default
  const shadow = data.highlighted
    ? "shadow-[0_0_16px_rgba(180,160,230,0.6)]"
    : data.hasHighFinding
      ? "shadow-[0_0_10px_rgba(218,54,51,0.5)]"
      : data.hasMediumFinding
        ? "shadow-[0_0_10px_rgba(210,153,34,0.4)]"
        : data.isSubject
          ? "shadow-[0_0_8px_rgba(180,160,230,0.3)]"
          : "";

  const border = data.highlighted
    ? "border-accent"
    : data.hasHighFinding
      ? "border-severity-high hover:border-severity-high hover:brightness-125"
      : data.hasMediumFinding
        ? "border-severity-medium hover:border-severity-medium hover:brightness-125"
        : data.isSubject
          ? "border-accent hover:brightness-125"
          : "border-text-muted/60 hover:border-text-secondary";

  return (
    <div
      title={data.modelId}
      className={clsx(
        "rounded border text-center cursor-pointer bg-bg-raised",
        "transition-all duration-300 ease-in-out",
        shadow,
        border,
        data.highlighted ? "px-3 py-2 scale-105" : "px-2.5 py-1.5",
        data.exists === false && "border-dashed border-severity-high/50",
      )}
    >
      <div className={clsx(
        "font-mono text-text-primary truncate max-w-40",
        data.highlighted ? "text-xs font-semibold" : "text-[11px]"
      )}>
        {shortId}
      </div>
      <div className="text-[9px] text-text-secondary truncate max-w-40">{data.modelId}</div>
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
