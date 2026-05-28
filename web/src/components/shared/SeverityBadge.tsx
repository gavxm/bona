import clsx from "clsx";
import type { Severity } from "../../types";

const styles: Record<Severity, string> = {
  high: "bg-severity-high text-white",
  medium: "bg-severity-medium text-bg-base",
  low: "bg-severity-low text-white",
  info: "bg-severity-info text-text-primary",
};

export function SeverityBadge({ severity }: { severity: Severity }) {
  return (
    <span
      className={clsx(
        "inline-block px-2 py-0.5 rounded text-xs font-bold uppercase tracking-wide",
        styles[severity]
      )}
    >
      {severity}
    </span>
  );
}
