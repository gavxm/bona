import { useState, useCallback, useRef, type ReactNode } from "react";
import type { ModelInvestigation } from "../types";
import { FINDING_LINKS, type CenterTab } from "../linking";
import { InvestigationContext } from "./investigationState";
import { decodePermalink } from "../permalink";

/** Parse permalink from the URL fragment at load time. This runs once,
 *  synchronously, before the first render so we can seed useState. */
function parseInitialPermalink() {
  const hash = window.location.hash;
  if (!hash.includes("s=")) return null;
  return decodePermalink(hash);
}

const initialPermalink = parseInitialPermalink();

function resolveInitialHighlights(inv: ModelInvestigation, findingId: string) {
  const link = FINDING_LINKS[findingId];
  if (!link) return { tab: "declared" as CenterTab, fields: [] as string[], graphNodes: [] as string[] };
  const graphNodes: string[] = [];
  for (const node of link.graphNodes) {
    if (node === "subject") graphNodes.push(inv.model_id);
    else if (node === "parent" && inv.lineage?.chain[0]) graphNodes.push(inv.lineage.chain[0].model_id);
  }
  return { tab: link.centerTab, fields: link.centerFields, graphNodes };
}

const initHighlights = initialPermalink?.findingId
  ? resolveInitialHighlights(initialPermalink.investigation, initialPermalink.findingId)
  : null;

export function InvestigationProvider({ children }: { children: ReactNode }) {
  const [investigation, setInvestigation] = useState<ModelInvestigation | null>(initialPermalink?.investigation ?? null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [selectedFindingId, setSelectedFindingId] = useState<string | null>(initialPermalink?.findingId ?? null);
  const [activeTab, setActiveTab] = useState<CenterTab>(initHighlights?.tab ?? "declared");
  const [highlightedFields, setHighlightedFields] = useState<string[]>(initHighlights?.fields ?? []);
  const [highlightedGraphNodes, setHighlightedGraphNodes] = useState<string[]>(initHighlights?.graphNodes ?? []);
  const [isSnapshot, setIsSnapshot] = useState(initialPermalink != null);
  const [schemaWarning, setSchemaWarning] = useState(initialPermalink?.versionMismatch ?? false);

  // Navigation history for back/forward between investigations.
  const historyBack = useRef<ModelInvestigation[]>([]);
  const historyForward = useRef<ModelInvestigation[]>([]);
  const [canGoBack, setCanGoBack] = useState(false);
  const [canGoForward, setCanGoForward] = useState(false);

  const selectFinding = useCallback(
    (id: string | null) => {
      if (id === selectedFindingId) {
        setSelectedFindingId(null);
        setHighlightedFields([]);
        setHighlightedGraphNodes([]);
        return;
      }

      setSelectedFindingId(id);

      if (id && FINDING_LINKS[id]) {
        const link = FINDING_LINKS[id];
        setActiveTab(link.centerTab);
        setHighlightedFields(link.centerFields);

        const graphNodes: string[] = [];
        if (investigation) {
          for (const node of link.graphNodes) {
            if (node === "subject") {
              graphNodes.push(investigation.model_id);
            } else if (node === "parent" && investigation.lineage?.chain[0]) {
              graphNodes.push(investigation.lineage.chain[0].model_id);
            }
          }
        }
        setHighlightedGraphNodes(graphNodes);
      } else {
        setHighlightedFields([]);
        setHighlightedGraphNodes([]);
      }
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

      // Apply finding selection inline instead of deferring through
      // selectFinding (which has unstable deps and would cause re-render loops).
      if (findingId && FINDING_LINKS[findingId]) {
        const link = FINDING_LINKS[findingId];
        setSelectedFindingId(findingId);
        setActiveTab(link.centerTab);
        setHighlightedFields(link.centerFields);
        const graphNodes: string[] = [];
        for (const node of link.graphNodes) {
          if (node === "subject") {
            graphNodes.push(inv.model_id);
          } else if (node === "parent" && inv.lineage?.chain[0]) {
            graphNodes.push(inv.lineage.chain[0].model_id);
          }
        }
        setHighlightedGraphNodes(graphNodes);
      } else {
        setSelectedFindingId(null);
        setHighlightedFields([]);
        setHighlightedGraphNodes([]);
      }
    },
    [],
  );

  const goBack = useCallback(() => {
    const prev = historyBack.current.pop();
    if (!prev) return;
    if (investigation) historyForward.current.push(investigation);
    setInvestigation(prev);
    setSelectedFindingId(null);
    setHighlightedFields([]);
    setHighlightedGraphNodes([]);
    setActiveTab("declared");
    setError(null);
    setIsSnapshot(false);
    setCanGoBack(historyBack.current.length > 0);
    setCanGoForward(true);
    const url = new URL(window.location.href);
    url.searchParams.set("model", prev.model_id);
    url.hash = "";
    window.history.replaceState(null, "", url.toString());
  }, [investigation]);

  const goForward = useCallback(() => {
    const next = historyForward.current.pop();
    if (!next) return;
    if (investigation) historyBack.current.push(investigation);
    setInvestigation(next);
    setSelectedFindingId(null);
    setHighlightedFields([]);
    setHighlightedGraphNodes([]);
    setActiveTab("declared");
    setError(null);
    setIsSnapshot(false);
    setCanGoBack(true);
    setCanGoForward(historyForward.current.length > 0);
    const url = new URL(window.location.href);
    url.searchParams.set("model", next.model_id);
    url.hash = "";
    window.history.replaceState(null, "", url.toString());
  }, [investigation]);

  const loadInvestigation = useCallback(async (modelId: string) => {
    // Push current investigation onto back stack inline (avoids unstable dep on pushHistory).
    setInvestigation((prev) => {
      if (prev) {
        historyBack.current.push(prev);
        historyForward.current = [];
        setCanGoBack(true);
        setCanGoForward(false);
      }
      return prev;
    });
    setLoading(true);
    setError(null);
    setIsSnapshot(false);
    setSelectedFindingId(null);
    setHighlightedFields([]);
    setHighlightedGraphNodes([]);
    setActiveTab("declared");

    try {
      const apiBase = import.meta.env.VITE_API_URL;
      let resp: Response | null = null;

      // Only attempt the API when an explicit URL is configured.
      if (apiBase) {
        resp = await fetch(`${apiBase}/api/investigate/${modelId}`).catch(
          () => null,
        );
        const isJson = resp?.headers
          .get("content-type")
          ?.includes("application/json");
        if (!resp || !resp.ok || !isJson) resp = null;
      }

      // Fall back to static gallery JSON.
      if (!resp) {
        const filename = modelId.replace("/", "--");
        const base = import.meta.env.BASE_URL;
        resp = await fetch(`${base}investigations/${filename}.json`);
        const isJson = resp.headers
          .get("content-type")
          ?.includes("application/json");
        if (!isJson) {
          throw new Error(
            `No investigation available for ${modelId}. Set VITE_API_URL to investigate live models.`,
          );
        }
      }
      if (!resp.ok)
        throw new Error(`Failed to load investigation for ${modelId}`);
      const data = await resp.json();
      setInvestigation(data);
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
  }, []);

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
      }}
    >
      {children}
    </InvestigationContext.Provider>
  );
}
