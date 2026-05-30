import { useMemo, useCallback } from "react";
import {
  ReactFlow,
  Background,
  useReactFlow,
  ReactFlowProvider,
  type Node,
  type Edge,
  type NodeMouseHandler,
  BackgroundVariant,
} from "@xyflow/react";
import { useInvestigation } from "../../context/useInvestigation";
import { FINDING_LINKS } from "../../linking";
import { ModelNode, type ModelNodeData } from "./ModelNode";
import { SiblingNode } from "./SiblingNode";

const nodeTypes = {
  model: ModelNode,
  sibling: SiblingNode,
};

function LineageGraphInner() {
  const { investigation, highlightedGraphNodes, selectFinding, loadInvestigation } =
    useInvestigation();
  const { fitView } = useReactFlow();

  const { nodes, edges } = useMemo(() => {
    if (!investigation) return { nodes: [], edges: [] };

    const nodes: Node[] = [];
    const edges: Edge[] = [];
    const lineage = investigation.lineage;
    let y = 20;

    // Compute which graph roles have findings.
    const nodeFindings = new Map<string, { high: boolean; medium: boolean }>();
    for (const f of investigation.findings) {
      const link = FINDING_LINKS[f.id];
      if (!link) continue;
      for (const role of link.graphNodes) {
        const id =
          role === "subject"
            ? investigation.model_id
            : role === "parent" && lineage?.chain[0]
              ? lineage.chain[0].model_id
              : null;
        if (!id) continue;
        const existing = nodeFindings.get(id) ?? { high: false, medium: false };
        if (f.severity === "high") existing.high = true;
        if (f.severity === "medium") existing.medium = true;
        nodeFindings.set(id, existing);
      }
    }

    // Ancestor chain nodes (most distant ancestor first, then parent).
    const chain = lineage?.chain ?? [];
    const reversedChain = [...chain].reverse();
    const nodeX = 70;
    for (const ancestor of reversedChain) {
      const af = nodeFindings.get(ancestor.model_id);
      nodes.push({
        id: ancestor.model_id,
        type: "model",
        position: { x: nodeX, y },
        data: {
          modelId: ancestor.model_id,
          license: ancestor.license,
          exists: ancestor.exists,
          isSubject: false,
          highlighted: highlightedGraphNodes.includes(ancestor.model_id),
          hasHighFinding: af?.high ?? false,
          hasMediumFinding: af?.medium ?? false,
        } satisfies ModelNodeData,
      });
      y += 120;
    }

    // Edges between chain nodes (ancestor -> child).
    for (let i = 0; i < reversedChain.length; i++) {
      const target =
        i < reversedChain.length - 1
          ? reversedChain[i + 1].model_id
          : investigation.model_id;
      edges.push({
        id: `${reversedChain[i].model_id}->${target}`,
        source: reversedChain[i].model_id,
        target,
        style: { stroke: "#8b949e", strokeWidth: 1.5 },
        animated: highlightedGraphNodes.includes(reversedChain[i].model_id),
      });
    }

    // Subject node
    const sf = nodeFindings.get(investigation.model_id);
    nodes.push({
      id: investigation.model_id,
      type: "model",
      position: { x: nodeX, y },
      data: {
        modelId: investigation.model_id,
        license: investigation.declared.declared_license,
        isSubject: true,
        highlighted: highlightedGraphNodes.includes(investigation.model_id),
        hasHighFinding: sf?.high ?? false,
        hasMediumFinding: sf?.medium ?? false,
      } satisfies ModelNodeData,
    });

    y += 120;

    // Sibling nodes - compact stack, max 3 shown
    const parentId = chain[0]?.model_id;
    if (lineage?.siblings && lineage.siblings.length > 0) {
      const visible = lineage.siblings.slice(0, 3);
      visible.forEach((sib, i) => {
        nodes.push({
          id: sib,
          type: "sibling",
          position: { x: nodeX + 20, y: y + i * 40 },
          data: { modelId: sib },
        });

        if (parentId) {
          edges.push({
            id: `${parentId}->${sib}`,
            source: parentId,
            target: sib,
            style: { stroke: "#6e7681" },
          });
        }
      });

      const remaining = lineage.siblings.length - visible.length;
      if (remaining > 0) {
        const overflowY = y + visible.length * 40;
        nodes.push({
          id: "__siblings_overflow",
          type: "sibling",
          position: { x: nodeX + 20, y: overflowY },
          data: { modelId: `+${remaining} more` },
        });
      }
    }

    return { nodes, edges };
  }, [investigation, highlightedGraphNodes]);

  const onNodeClick: NodeMouseHandler = useCallback(
    (_event, node) => {
      if (!investigation) return;

      if (node.id === investigation.model_id) {
        selectFinding(null);
        return;
      }

      // Ignore the overflow placeholder.
      if (node.id === "__siblings_overflow") return;

      // Pivot to investigate the clicked model.
      loadInvestigation(node.id);
    },
    [investigation, selectFinding, loadInvestigation]
  );

  if (!investigation) return null;

  return (
    <div className="h-full w-full bg-bg-surface border-r border-border">
      <div className="px-3 py-3 border-b border-border flex items-center justify-between">
        <h2 className="text-xs font-semibold text-text-secondary uppercase tracking-wide">
          Lineage
        </h2>
        <button
          onClick={() => fitView({ padding: 0.3, duration: 200 })}
          className="text-[10px] text-text-muted hover:text-text-secondary cursor-pointer px-1.5 py-0.5 rounded border border-border hover:border-text-muted transition-colors"
          title="Reset"
        >
          Reset
        </button>
      </div>
      <div className="h-[calc(100%-40px)]">
        <ReactFlow
          nodes={nodes}
          edges={edges}
          nodeTypes={nodeTypes}
          onNodeClick={onNodeClick}
          fitView
          fitViewOptions={{ padding: 0.3 }}
          proOptions={{ hideAttribution: true }}
          nodesDraggable={false}
          nodesConnectable={false}
          panOnDrag
          zoomOnScroll
          minZoom={0.5}
          maxZoom={2}
        >
          <Background variant={BackgroundVariant.Dots} color="#30363d" size={1} gap={20} />
        </ReactFlow>
      </div>
    </div>
  );
}

export function LineageGraph() {
  return (
    <ReactFlowProvider>
      <LineageGraphInner />
    </ReactFlowProvider>
  );
}
