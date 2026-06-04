import { useMemo } from "react";
import { FINDING_LINKS } from "../../linking";
import type { Finding } from "../../types";

/** Compute which fields have findings pointing at them and the highest severity. */
export function useFieldFlags(findings: Finding[]): Map<string, "high" | "medium"> {
  return useMemo(() => {
    const flags = new Map<string, "high" | "medium">();
    for (const f of findings) {
      const link = FINDING_LINKS[f.id];
      if (!link) continue;
      const sev = f.severity === "high" ? "high" : f.severity === "medium" ? "medium" : null;
      if (!sev) continue;
      for (const field of link.centerFields) {
        const existing = flags.get(field);
        if (!existing || (sev === "high" && existing === "medium")) {
          flags.set(field, sev);
        }
      }
    }
    return flags;
  }, [findings]);
}
