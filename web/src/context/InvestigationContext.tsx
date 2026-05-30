import { useState, useCallback, type ReactNode } from "react";
import type { ModelInvestigation } from "../types";
import { FINDING_LINKS, type CenterTab } from "../linking";
import { InvestigationContext } from "./investigationState";

export function InvestigationProvider({ children }: { children: ReactNode }) {
  const [investigation, setInvestigation] = useState<ModelInvestigation | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [selectedFindingId, setSelectedFindingId] = useState<string | null>(null);
  const [activeTab, setActiveTab] = useState<CenterTab>("declared");
  const [highlightedFields, setHighlightedFields] = useState<string[]>([]);
  const [highlightedGraphNodes, setHighlightedGraphNodes] = useState<string[]>([]);

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

  const loadInvestigation = useCallback(async (modelId: string) => {
    setLoading(true);
    setError(null);
    setSelectedFindingId(null);
    setHighlightedFields([]);
    setHighlightedGraphNodes([]);
    setActiveTab("declared");

    try {
      const apiBase = import.meta.env.VITE_API_URL ?? "";
      let resp = await fetch(`${apiBase}/api/investigate/${modelId}`).catch(() => null);
      const isJson = resp?.headers.get("content-type")?.includes("application/json");

      if (!resp || !resp.ok || !isJson) {
        const filename = modelId.replace("/", "--");
        const base = import.meta.env.BASE_URL;
        resp = await fetch(`${base}investigations/${filename}.json`);
        const fallbackIsJson = resp.headers.get("content-type")?.includes("application/json");
        if (!fallbackIsJson) {
          throw new Error(`No investigation available for ${modelId}. Start the API server to investigate live models.`);
        }
      }
      if (!resp.ok) throw new Error(`Failed to load investigation for ${modelId}`);
      const data = await resp.json();
      setInvestigation(data);
      const url = new URL(window.location.href);
      url.searchParams.set("model", modelId);
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
        selectFinding,
        setActiveTab,
        loadInvestigation,
      }}
    >
      {children}
    </InvestigationContext.Provider>
  );
}
