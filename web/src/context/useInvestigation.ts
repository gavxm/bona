import { useContext } from "react";
import { InvestigationContext } from "./investigationState";

export function useInvestigation() {
  const ctx = useContext(InvestigationContext);
  if (!ctx) throw new Error("useInvestigation must be inside InvestigationProvider");
  return ctx;
}
