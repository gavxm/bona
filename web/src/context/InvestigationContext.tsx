import { useState, useCallback, useRef, useMemo, useEffect, type ReactNode } from "react";
import type { ModelInvestigation, Finding } from "../types";
import { FINDING_LINKS, extractModelIdFromFinding, type CenterTab, type GraphNodeRef } from "../linking";
import { InvestigationContext } from "./investigationState";
import { decodePermalink } from "../permalink";

/* ---- Helpers ---- */

function resolveGraphNodes(
  refs: GraphNodeRef[],
  inv: ModelInvestigation,
  finding?: Finding,
): string[] {
  const nodes: string[] = [];
  for (const ref of refs) {
    if (ref === "subject") {
      nodes.push(inv.model_id);
    } else if (ref === "parent" && inv.lineage?.chain[0]) {
      nodes.push(inv.lineage.chain[0].model_id);
    } else if (typeof ref === "object" && ref.fromFinding && finding?.evidence_url) {
      const id = extractModelIdFromFinding(finding.evidence_url);
      if (id) nodes.push(id);
    }
  }
  return nodes;
}

function resolveHighlights(inv: ModelInvestigation, findingId: string) {
  const link = FINDING_LINKS[findingId];
  if (!link) return null;
  const finding = inv.findings.find((f) => f.id === findingId);
  return {
    tab: link.centerTab,
    fields: link.centerFields,
    graphNodes: resolveGraphNodes(link.graphNodes, inv, finding),
  };
}

/* ---- Provider ---- */

export function InvestigationProvider({ children }: { children: ReactNode }) {
  // #1: Lazy initializer. Permalink parsing runs once on mount, not at import time.
  const [initial] = useState(() => {
    const hash = window.location.hash;
    if (!hash.includes("s=")) return null;
    const result = decodePermalink(hash);
    if (result) document.title = `${result.investigation.model_id} - yurai`;
    return result;
  });

  const initHighlights = initial?.findingId
    ? resolveHighlights(initial.investigation, initial.findingId)
    : null;

  const [investigation, setInvestigation] = useState<ModelInvestigation | null>(initial?.investigation ?? null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [selectedFindingId, setSelectedFindingId] = useState<string | null>(initial?.findingId ?? null);
  const [activeTab, setActiveTab] = useState<CenterTab>(initHighlights?.tab ?? "declared");
  const [highlightedFields, setHighlightedFields] = useState<string[]>(initHighlights?.fields ?? []);
  const [highlightedGraphNodes, setHighlightedGraphNodes] = useState<string[]>(initHighlights?.graphNodes ?? []);
  const [isSnapshot, setIsSnapshot] = useState(initial != null);
  const [schemaWarning, setSchemaWarning] = useState(initial?.versionMismatch ?? false);

  // Ref tracks current investigation for stable callbacks.
  const investigationRef = useRef(investigation);
  useEffect(() => { investigationRef.current = investigation; }, [investigation]);

  // Navigation history.
  const historyBack = useRef<ModelInvestigation[]>([]);
  const historyForward = useRef<ModelInvestigation[]>([]);
  const [canGoBack, setCanGoBack] = useState(false);
  const [canGoForward, setCanGoForward] = useState(false);

  const [focusedNode, setFocusedNodeRaw] = useState<string | null>(null);
  const [pulseKey, setPulseKey] = useState(0);

  // Reverse highlight: findings that reference the focused node.
  const relatedFindings = useMemo(() => {
    if (!focusedNode || !investigation) return new Set<string>();
    const related = new Set<string>();
    for (const f of investigation.findings) {
      const link = FINDING_LINKS[f.id];
      if (!link) continue;
      const resolved = resolveGraphNodes(link.graphNodes, investigation, f);
      if (resolved.includes(focusedNode)) related.add(f.id);
    }
    return related;
  }, [focusedNode, investigation]);

  // #2: Shared reset helper used by goBack, goForward, loadInvestigation.
  const clearHighlights = useCallback(() => {
    setSelectedFindingId(null);
    setHighlightedFields([]);
    setHighlightedGraphNodes([]);
    setFocusedNodeRaw(null);
    setActiveTab("declared");
  }, []);

  /** Show a new investigation and update URL. Used by goBack/goForward. */
  const showInvestigation = useCallback((inv: ModelInvestigation) => {
    setInvestigation(inv);
    clearHighlights();
    setError(null);
    setIsSnapshot(false);
    document.title = `${inv.model_id} - yurai`;
    const url = new URL(window.location.href);
    url.searchParams.set("model", inv.model_id);
    url.hash = "";
    window.history.replaceState(null, "", url.toString());
  }, [clearHighlights]);

  const setFocusedNode = useCallback(
    (id: string | null) => {
      setFocusedNodeRaw(id);
      if (id) {
        setSelectedFindingId(null);
        setHighlightedFields([]);
        setHighlightedGraphNodes([]);
      }
    },
    [],
  );

  const selectFinding = useCallback(
    (id: string | null) => {
      if (id === selectedFindingId) {
        setSelectedFindingId(null);
        setHighlightedFields([]);
        setHighlightedGraphNodes([]);
        return;
      }

      setSelectedFindingId(id);
      setFocusedNodeRaw(null);
      setPulseKey((k) => k + 1);

      if (id && investigation) {
        const hl = resolveHighlights(investigation, id);
        if (hl) {
          setActiveTab(hl.tab);
          setHighlightedFields(hl.fields);
          setHighlightedGraphNodes(hl.graphNodes);
          return;
        }
      }
      setHighlightedFields([]);
      setHighlightedGraphNodes([]);
    },
    [selectedFindingId, investigation]
  );

  const setInvestigationDirect = useCallback(
    (inv: ModelInvestigation, findingId?: string | null) => {
      setLoading(false);
      setError(null);
      setInvestigation(inv);
      setIsSnapshot(true);
      setActiveTab("declared");

      if (findingId) {
        const hl = resolveHighlights(inv, findingId);
        if (hl) {
          setSelectedFindingId(findingId);
          setActiveTab(hl.tab);
          setHighlightedFields(hl.fields);
          setHighlightedGraphNodes(hl.graphNodes);
          return;
        }
      }
      setSelectedFindingId(null);
      setHighlightedFields([]);
      setHighlightedGraphNodes([]);
    },
    [],
  );

  const goBack = useCallback(() => {
    const prev = historyBack.current.pop();
    if (!prev) return;
    const current = investigationRef.current;
    if (current) historyForward.current.push(current);
    showInvestigation(prev);
    setCanGoBack(historyBack.current.length > 0);
    setCanGoForward(true);
  }, [showInvestigation]);

  const goForward = useCallback(() => {
    const next = historyForward.current.pop();
    if (!next) return;
    const current = investigationRef.current;
    if (current) historyBack.current.push(current);
    showInvestigation(next);
    setCanGoBack(true);
    setCanGoForward(historyForward.current.length > 0);
  }, [showInvestigation]);

  const loadInvestigation = useCallback(async (modelId: string) => {
    const current = investigationRef.current;
    if (current) {
      historyBack.current.push(current);
      historyForward.current = [];
      setCanGoBack(true);
      setCanGoForward(false);
    }

    setLoading(true);
    setError(null);
    setIsSnapshot(false);
    clearHighlights();

    try {
      const apiBase = import.meta.env.VITE_API_URL;
      let resp: Response | null = null;

      if (apiBase) {
        resp = await fetch(`${apiBase}/api/investigate/${modelId}`).catch(() => null);
        const isJson = resp?.headers.get("content-type")?.includes("application/json");
        if (!resp || !resp.ok || !isJson) resp = null;
      }

      if (!resp) {
        const filename = modelId.replace("/", "--");
        const base = import.meta.env.BASE_URL;
        resp = await fetch(`${base}investigations/${filename}.json`);
        const isJson = resp.headers.get("content-type")?.includes("application/json");
        if (!isJson) {
          throw new Error(`No investigation available for ${modelId}. Set VITE_API_URL to investigate live models.`);
        }
      }
      if (!resp.ok) throw new Error(`Failed to load investigation for ${modelId}`);
      const data = await resp.json();
      setInvestigation(data);
      document.title = `${modelId} - yurai`;
      const url = new URL(window.location.href);
      url.searchParams.set("model", modelId);
      url.hash = "";
      window.history.replaceState(null, "", url.toString());
    } catch (e) {
      setError(e instanceof Error ? e.message : "Unknown error");
      setInvestigation(null);
    } finally {
      setLoading(false);
    }
  }, [clearHighlights]);

  return (
    <InvestigationContext.Provider
      value={{
        investigation,
        loading,
        error,
        selectedFindingId,
        activeTab,
        highlightedFields,
        highlightedGraphNodes,
        isSnapshot,
        schemaWarning,
        dismissSchemaWarning: () => setSchemaWarning(false),
        selectFinding,
        setActiveTab,
        loadInvestigation,
        setInvestigationDirect,
        canGoBack,
        canGoForward,
        goBack,
        goForward,
        focusedNode,
        setFocusedNode,
        relatedFindings,
        pulseKey,
      }}
    >
      {children}
    </InvestigationContext.Provider>
  );
}
