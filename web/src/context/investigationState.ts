import { createContext } from "react";
import type { ModelInvestigation } from "../types";
import type { CenterTab } from "../linking";

export interface InvestigationState {
  investigation: ModelInvestigation | null;
  loading: boolean;
  error: string | null;
  selectedFindingId: string | null;
  activeTab: CenterTab;
  highlightedFields: string[];
  highlightedGraphNodes: string[];
  selectFinding: (id: string | null) => void;
  setActiveTab: (tab: CenterTab) => void;
  loadInvestigation: (modelId: string) => Promise<void>;
}

export const InvestigationContext = createContext<InvestigationState | null>(null);
