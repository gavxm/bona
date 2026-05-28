import { Handle, Position, type NodeProps } from "@xyflow/react";

export interface SiblingNodeData {
  modelId: string;
}

export function SiblingNode({ data }: NodeProps) {
  const d = data as unknown as SiblingNodeData;
  const shortId = d.modelId.split("/").pop() ?? d.modelId;

  return (
    <div className="px-2 py-1 rounded border border-border bg-bg-surface text-center cursor-pointer hover:border-text-muted transition-colors">
      <div className="text-[10px] font-mono text-text-muted truncate max-w-30">
        {shortId} ↗
      </div>
      <Handle type="target" position={Position.Top} className="bg-text-muted! w-1! h-1! border-0!" />
    </div>
  );
}
