import { useEffect } from "react";
import { InvestigationProvider } from "./context/InvestigationContext";
import { useInvestigation } from "./context/useInvestigation";
import { AppShell } from "./components/AppShell";

function AppInner() {
  const { loadInvestigation } = useInvestigation();

  useEffect(() => {
    const params = new URLSearchParams(window.location.search);
    const model = params.get("model");
    if (model) {
      loadInvestigation(model);
    }
  }, [loadInvestigation]);

  return <AppShell />;
}

export default function App() {
  return (
    <InvestigationProvider>
      <AppInner />
    </InvestigationProvider>
  );
}
