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
  isSnapshot: boolean;
  schemaWarning: boolean;
  dismissSchemaWarning: () => void;
  selectFinding: (id: string | null) => void;
  setActiveTab: (tab: CenterTab) => void;
  loadInvestigation: (modelId: string) => Promise<void>;
  setInvestigationDirect: (inv: ModelInvestigation, findingId?: string | null) => void;
  canGoBack: boolean;
  canGoForward: boolean;
  goBack: () => void;
  goForward: () => void;
  focusedNode: string | null;
  setFocusedNode: (id: string | null) => void;
  relatedFindings: Set<string>;
  pulseKey: number;
}

export const InvestigationContext = createContext<InvestigationState | null>(null);
