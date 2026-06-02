import { useMemo, useCallback } from "react";
import { useInvestigation } from "../../context/useInvestigation";
import { FINDING_LINKS } from "../../linking";
import { IconLock } from "../shared/Icons";
import type { LineageNode } from "../../types";

/* ---- helpers ---- */

function depthLabel(depth: number): string {
  if (depth === 0) return "parent";
  if (depth === 1) return "grandparent";
  if (depth === 2) return "great-grandparent";
  return `ancestor (depth ${depth + 1})`;
}

const RELATION_LABELS: Record<string, string> = {
  finetune: "fine-tune",
  quantization: "quantized",
  merge: "merge",
  adapter: "adapter",
};

/* ---- sub-components ---- */

interface GutterProps {
  topEdge?: { dashed?: boolean; id: string };
  botEdge?: { dashed?: boolean; id: string };
  dotColor: string;
  glow: boolean;
  dim: boolean;
  glowEdges: Set<string>;
}

function Gutter({ topEdge, botEdge, dotColor, glow, dim, glowEdges }: GutterProps) {
  const segClass = (edge?: { dashed?: boolean; id: string }) => {
    if (!edge) return "hidden";
    const on = glowEdges.has(edge.id);
    return [
      "absolute left-1/2 -translate-x-1/2 w-[1.6px]",
      edge.dashed ? "border-l-[1.6px] border-dashed border-border-strong w-0 bg-transparent" : "bg-border-strong",
      on ? "!bg-accent !border-accent" : "",
      dim && !on ? "opacity-30" : "",
    ].join(" ");
  };

  return (
    <div className="relative w-4 shrink-0">
      {topEdge && <div className={`${segClass(topEdge)} top-0 h-1/2`} />}
      {botEdge && <div className={`${segClass(botEdge)} top-1/2 bottom-0`} />}
      <span
        className={[
          "absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-[11px] h-[11px] rounded-full z-[2] border-[2.5px] border-bg-surface transition-opacity duration-200",
          dim && !glow ? "opacity-30" : "",
        ].join(" ")}
        style={{ background: dotColor }}
      />
    </div>
  );
}

interface NodeCardProps {
  name: string;
  repo: string;
  note?: string;
  tags: { label: string; warn?: boolean }[];
  isSelf: boolean;
  gated?: boolean;
  glow: boolean;
  focused: boolean;
  dim: boolean;
  onClick: () => void;
}

function NodeCard({ name, repo, note, tags, isSelf, gated, glow, focused, dim, onClick }: NodeCardProps) {
  return (
    <div
      onClick={onClick}
      className={[
        "flex-1 min-w-0 my-1.5 px-3.5 py-3 rounded-xl cursor-pointer",
        "bg-bg-raised border transition-all duration-200",
        isSelf ? "border-accent-line shadow-[inset_0_0_0_1px_var(--color-accent-line)]" : "border-border hover:border-border-strong",
        glow ? "shadow-[0_0_0_1px_var(--color-accent-line),0_0_22px_-2px_rgba(155,131,223,0.55)] border-accent-line" : "",
        focused ? "shadow-[0_0_0_1px_var(--color-accent-line),0_0_22px_-2px_rgba(155,131,223,0.55)] border-accent" : "",
        dim ? "opacity-[0.34]" : "",
        !dim && !glow && !focused ? "hover:-translate-y-px" : "",
      ].join(" ")}
    >
      <div className="text-[13px] font-semibold text-text-primary tracking-tight flex items-center gap-1.5">
        {gated && <IconLock size={11} className="text-severity-medium" />}
        {name}
      </div>
      <div className="font-mono text-[10px] text-text-muted mt-0.5 truncate">{repo}</div>
      {note && <div className="text-[10.5px] text-text-secondary mt-1.5 italic">{note}</div>}
      {tags.length > 0 && (
        <div className="flex gap-1 mt-2 flex-wrap">
          {tags.map((t, i) => (
            <span
              key={i}
              className={[
                "font-mono text-[9px] tracking-wide px-1.5 py-0.5 rounded border inline-flex items-center gap-1",
                t.warn
                  ? "text-severity-medium border-severity-medium-line bg-severity-medium-bg"
                  : "text-text-muted border-border bg-bg-base",
              ].join(" ")}
            >
              {t.label}
            </span>
          ))}
        </div>
      )}
    </div>
  );
}

interface SiblingPillProps {
  name: string;
  label: string;
  dim: boolean;
  onClick: () => void;
}

function SiblingPill({ name, label, dim, onClick }: SiblingPillProps) {
  return (
    <div
      onClick={onClick}
      className={[
        "flex items-center gap-2 px-2.5 py-1.5 rounded-lg bg-bg-raised border border-dashed border-border-strong cursor-pointer transition-all duration-200",
        "hover:border-solid hover:border-border-strong",
        dim ? "opacity-40" : "",
      ].join(" ")}
    >
      <span className="w-1.5 h-1.5 rounded-full bg-text-muted shrink-0" />
      <span className="text-[11.5px] text-text-secondary truncate">{name}</span>
      <span className="font-mono text-[8.5px] text-text-muted ml-auto shrink-0">{label}</span>
    </div>
  );
}

/* ---- main component ---- */

export function LineageGraph() {
  const {
    investigation,
    selectedFindingId,
    highlightedGraphNodes,
    focusedNode,
    setFocusedNode,
    loadInvestigation,
  } = useInvestigation();

  // Build glow sets from the current selection.
  const { glowNodes, glowEdges, dimNonGlow } = useMemo(() => {
    const glowNodes = new Set(highlightedGraphNodes);
    const glowEdges = new Set<string>();
    if (focusedNode) glowNodes.add(focusedNode);

    // Compute glowing edges from the selected finding.
    if (selectedFindingId && investigation) {
      const finding = investigation.findings.find((f) => f.id === selectedFindingId);
      const link = FINDING_LINKS[selectedFindingId];
      if (link && finding) {
        const resolvedNodes = new Set(highlightedGraphNodes);
        // Build edges between consecutive glow nodes in the chain.
        const chain = investigation.lineage?.chain ?? [];
        const allIds = [...chain.map((n) => n.model_id).reverse(), investigation.model_id];
        for (let i = 0; i < allIds.length - 1; i++) {
          if (resolvedNodes.has(allIds[i]) || resolvedNodes.has(allIds[i + 1])) {
            glowEdges.add(`${allIds[i]}->${allIds[i + 1]}`);
          }
        }
      }
    }

    return {
      glowNodes,
      glowEdges,
      dimNonGlow: !!selectedFindingId || !!focusedNode,
    };
  }, [highlightedGraphNodes, selectedFindingId, focusedNode, investigation]);

  const handleNodeClick = useCallback(
    (nodeId: string) => {
      if (!investigation) return;
      if (nodeId === investigation.model_id) {
        // Clicking subject clears everything.
        setFocusedNode(null);
        return;
      }
      setFocusedNode(focusedNode === nodeId ? null : nodeId);
    },
    [investigation, focusedNode, setFocusedNode],
  );

  if (!investigation) return null;

  const lineage = investigation.lineage;
  const chain = lineage?.chain ?? [];
  const reversedChain = [...chain].reverse();
  const siblings = lineage?.siblings ?? [];

  // Build node data for rendering.
  const isGlow = (id: string) => glowNodes.has(id);
  const isDim = (id: string) => dimNonGlow && !glowNodes.has(id);
  const isFocused = (id: string) => focusedNode === id;

  // Build edges list for gutter.
  const edgeIds: { from: string; to: string; dashed: boolean; id: string }[] = [];
  for (let i = 0; i < reversedChain.length; i++) {
    const from = reversedChain[i].model_id;
    const to = i < reversedChain.length - 1 ? reversedChain[i + 1].model_id : investigation.model_id;
    edgeIds.push({ from, to, dashed: false, id: `${from}->${to}` });
  }

  // Determine dot colors.
  function dotColor(node: LineageNode): string {
    if (node.gated === "auto" || node.gated === "manual") return "var(--color-severity-medium)";
    return "var(--color-severity-low)";
  }

  // Tags for ancestor nodes.
  function ancestorTags(node: LineageNode): { label: string; warn: boolean }[] {
    const tags: { label: string; warn: boolean }[] = [];
    if (node.gated === "auto" || node.gated === "manual") {
      tags.push({ label: "gated", warn: true });
    }
    if (node.license) tags.push({ label: node.license, warn: false });
    const rel = RELATION_LABELS[node.relation];
    if (rel) tags.push({ label: rel, warn: false });
    return tags;
  }

  return (
    <div className="h-full w-full bg-bg-surface border-r border-border flex flex-col">
      {/* Header */}
      <div className="px-5 py-3 border-b border-border flex items-center gap-2.5">
        <span className="font-mono text-[11px] font-semibold tracking-widest uppercase text-text-secondary">
          Lineage
        </span>
        <span className="font-mono text-[10.5px] text-text-muted">
          {reversedChain.length + 1} nodes{reversedChain.length > 0 ? ` - depth ${reversedChain.length}` : ""}
        </span>
        <span className="flex-1" />
        {focusedNode && (
          <button
            onClick={() => setFocusedNode(null)}
            className="text-[11px] text-text-muted hover:text-text-secondary cursor-pointer px-2 py-0.5 rounded border border-border hover:border-border-strong transition-colors"
          >
            reset
          </button>
        )}
      </div>

      {/* Spine */}
      <div className="flex-1 min-h-0 overflow-y-auto scroll px-5 py-4">
        <div className="flex flex-col">
          {/* Ancestor chain */}
          {reversedChain.map((ancestor, i) => {
            const topEdge = i > 0 ? edgeIds[i - 1] : undefined;
            const botEdge = edgeIds[i];
            return (
              <div key={ancestor.model_id} className="flex items-stretch gap-3">
                <Gutter
                  topEdge={topEdge}
                  botEdge={botEdge}
                  dotColor={dotColor(ancestor)}
                  glow={isGlow(ancestor.model_id)}
                  dim={isDim(ancestor.model_id)}
                  glowEdges={glowEdges}
                />
                <NodeCard
                  name={ancestor.model_id.split("/").pop() ?? ancestor.model_id}
                  repo={ancestor.model_id}
                  note={depthLabel(ancestor.depth)}
                  tags={ancestorTags(ancestor)}
                  isSelf={false}
                  gated={ancestor.gated === "auto" || ancestor.gated === "manual"}
                  glow={isGlow(ancestor.model_id)}
                  focused={isFocused(ancestor.model_id)}
                  dim={isDim(ancestor.model_id)}
                  onClick={() => handleNodeClick(ancestor.model_id)}
                />
              </div>
            );
          })}

          {/* Siblings cluster (between parent and subject) */}
          {siblings.length > 0 && reversedChain.length > 0 && (
            <div className="flex items-stretch gap-3">
              <Gutter
                topEdge={edgeIds[edgeIds.length - 1]}
                botEdge={edgeIds[edgeIds.length - 1]}
                dotColor="transparent"
                glow={false}
                dim={dimNonGlow}
                glowEdges={glowEdges}
              />
              <div className="flex-1 min-w-0 my-0.5 pl-1">
                <div className="font-mono text-[9px] tracking-widest uppercase text-text-muted mb-1">
                  siblings
                </div>
                <div className="flex flex-col gap-1">
                  {siblings.slice(0, 4).map((sib) => (
                    <SiblingPill
                      key={sib}
                      name={sib.split("/").pop() ?? sib}
                      label={sib.split("/")[0] ?? ""}
                      dim={isDim(sib)}
                      onClick={() => loadInvestigation(sib)}
                    />
                  ))}
                  {siblings.length > 4 && (
                    <span className="text-[10px] text-text-muted pl-4">+{siblings.length - 4} more</span>
                  )}
                </div>
              </div>
            </div>
          )}

          {/* Subject node */}
          <div className="flex items-stretch gap-3">
            <Gutter
              topEdge={edgeIds.length > 0 ? edgeIds[edgeIds.length - 1] : undefined}
              botEdge={undefined}
              dotColor="var(--color-accent)"
              glow={isGlow(investigation.model_id)}
              dim={isDim(investigation.model_id)}
              glowEdges={glowEdges}
            />
            <NodeCard
              name={investigation.model_id.split("/").pop() ?? investigation.model_id}
              repo={investigation.model_id}
              note="investigated"
              tags={[
                { label: "this model", warn: false },
                ...(investigation.declared.declared_license
                  ? [{ label: investigation.declared.declared_license, warn: false }]
                  : []),
              ]}
              isSelf={true}
              glow={isGlow(investigation.model_id)}
              focused={isFocused(investigation.model_id)}
              dim={isDim(investigation.model_id)}
              onClick={() => handleNodeClick(investigation.model_id)}
            />
          </div>
        </div>
      </div>

      {/* Legend */}
      <div className="flex gap-3.5 flex-wrap px-5 py-2.5 border-t border-border font-mono text-[9.5px] text-text-muted">
        <span className="flex items-center gap-1.5">
          <i className="inline-block w-[7px] h-[7px] rounded-full" style={{ background: "var(--color-accent)" }} />
          this model
        </span>
        <span className="flex items-center gap-1.5">
          <i className="inline-block w-[7px] h-[7px] rounded-full" style={{ background: "var(--color-severity-medium)" }} />
          gated origin
        </span>
      </div>
    </div>
  );
}
