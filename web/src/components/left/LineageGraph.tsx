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
  const { investigation, highlightedGraphNodes, selectFinding, setActiveTab } =
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
            : role === "parent" && lineage?.parent_id
              ? lineage.parent_id
              : null;
        if (!id) continue;
        const existing = nodeFindings.get(id) ?? { high: false, medium: false };
        if (f.severity === "high") existing.high = true;
        if (f.severity === "medium") existing.medium = true;
        nodeFindings.set(id, existing);
      }
    }

    // Parent node
    if (lineage?.parent_id) {
      const pf = nodeFindings.get(lineage.parent_id);
      nodes.push({
        id: lineage.parent_id,
        type: "model",
        position: { x: 40, y },
        data: {
          modelId: lineage.parent_id,
          license: lineage.parent_license,
          exists: lineage.parent_exists,
          isSubject: false,
          highlighted: highlightedGraphNodes.includes(lineage.parent_id),
          hasHighFinding: pf?.high ?? false,
          hasMediumFinding: pf?.medium ?? false,
        } satisfies ModelNodeData,
      });

      edges.push({
        id: `${lineage.parent_id}->${investigation.model_id}`,
        source: lineage.parent_id,
        target: investigation.model_id,
        style: { stroke: "#484f58" },
        animated: highlightedGraphNodes.includes(lineage.parent_id),
      });

      y += 120;
    }

    // Subject node
    const sf = nodeFindings.get(investigation.model_id);
    nodes.push({
      id: investigation.model_id,
      type: "model",
      position: { x: 40, y },
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

    // Sibling nodes - stacked vertically
    if (lineage?.siblings) {
      lineage.siblings.forEach((sib, i) => {
        nodes.push({
          id: sib,
          type: "sibling",
          position: { x: 60, y: y + i * 50 },
          data: { modelId: sib },
        });

        if (lineage.parent_id) {
          edges.push({
            id: `${lineage.parent_id}->${sib}`,
            source: lineage.parent_id,
            target: sib,
            style: { stroke: "#30363d" },
          });
        }
      });
    }

    return { nodes, edges };
  }, [investigation, highlightedGraphNodes]);

  const onNodeClick: NodeMouseHandler = useCallback(
    (_event, node) => {
      if (!investigation) return;

      if (node.type === "sibling") {
        // Open sibling on HuggingFace
        window.open(`https://huggingface.co/${node.id}`, "_blank");
        return;
      }

      if (node.id === investigation.model_id) {
        // Subject node - deselect any finding
        selectFinding(null);
        return;
      }

      // Parent node - show declared tab with license highlighted
      if (node.id === investigation.lineage?.parent_id) {
        setActiveTab("declared");
      }
    },
    [investigation, selectFinding, setActiveTab]
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
