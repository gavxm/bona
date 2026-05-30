import { useEffect } from "react";
import { InvestigationProvider } from "./context/InvestigationContext";
import { useInvestigation } from "./context/useInvestigation";
import { AppShell } from "./components/AppShell";

function AppInner() {
  const { loadInvestigation, isSnapshot, schemaWarning, dismissSchemaWarning } =
    useInvestigation();

  useEffect(() => {
    // Permalink snapshots are handled synchronously in the provider's
    // initial state. Only the ?model= query param needs an effect.
    if (isSnapshot) return;

    const params = new URLSearchParams(window.location.search);
    const model = params.get("model");
    if (model) {
      loadInvestigation(model);
    }
  }, [loadInvestigation, isSnapshot]);

  return (
    <>
      {schemaWarning && (
        <div className="flex items-center justify-between px-4 py-1.5 bg-severity-medium/10 border-b border-severity-medium/30 text-[11px] text-severity-medium">
          <span>This permalink was created with an older schema version. Some fields may not display correctly.</span>
          <button
            onClick={dismissSchemaWarning}
            className="text-severity-medium hover:text-text-primary cursor-pointer ml-4 shrink-0"
          >
            dismiss
          </button>
        </div>
      )}
      <AppShell />
    </>
  );
}

export default function App() {
  return (
    <InvestigationProvider>
      <AppInner />
    </InvestigationProvider>
  );
}
