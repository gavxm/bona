import { Handle, Position, type NodeProps, type Node } from "@xyflow/react";

export type SiblingNodeData = {
  modelId: string;
};

export type SiblingNodeType = Node<SiblingNodeData, "sibling">;

export function SiblingNode({ data }: NodeProps<SiblingNodeType>) {
  const shortId = data.modelId.split("/").pop() ?? data.modelId;

  return (
    <div className="px-2 py-1 rounded border border-text-muted/40 bg-bg-raised text-center cursor-pointer hover:border-text-secondary transition-colors">
      <div className="text-[10px] font-mono text-text-secondary truncate max-w-30">
        {shortId} ↗
      </div>
      <Handle type="target" position={Position.Top} className="bg-text-muted! w-1! h-1! border-0!" />
    </div>
  );
}
