import { useEffect, useRef } from "react";
import { useInvestigation } from "../../context/useInvestigation";

interface FieldRowProps {
  label: string;
  field: string;
  value: string | null | undefined;
  mono?: boolean;
  flag?: "high" | "medium" | null;
}

const FLAG_STYLES = {
  high: "bg-severity-high-bg text-severity-high",
  medium: "bg-severity-medium-bg text-severity-medium",
};

export function FieldRow({ label, field, value, mono = false, flag }: FieldRowProps) {
  const { highlightedFields, pulseKey } = useInvestigation();
  const isHighlighted = highlightedFields.includes(field);
  const ref = useRef<HTMLDivElement>(null);
  const prevPulseKey = useRef(pulseKey);

  useEffect(() => {
    if (isHighlighted && ref.current) {
      ref.current.scrollIntoView({ behavior: "smooth", block: "nearest" });
    }
  }, [isHighlighted]);

  // Re-trigger pulse animation on pulseKey change.
  useEffect(() => {
    if (isHighlighted && ref.current && pulseKey !== prevPulseKey.current) {
      ref.current.classList.remove("animate-fieldpulse");
      // Force reflow to restart animation.
      void ref.current.offsetWidth;
      ref.current.classList.add("animate-fieldpulse");
    }
    prevPulseKey.current = pulseKey;
  }, [pulseKey, isHighlighted]);

  return (
    <div
      ref={ref}
      className={[
        "grid items-baseline gap-x-4 px-2.5 py-2.5 rounded-lg -mx-2.5 transition-all duration-300 border-t border-border first:border-t-0",
        isHighlighted ? "bg-accent-bg shadow-[inset_0_0_0_1px_var(--color-accent-line)]" : "",
      ].join(" ")}
      style={{ gridTemplateColumns: "160px minmax(0, 1fr)" }}
    >
      <span className="text-text-muted text-[12.5px] flex items-center gap-1.5">
        {flag && (
          <span className={`inline-flex items-center justify-center w-3.5 h-3.5 rounded text-[9px] font-mono font-bold ${FLAG_STYLES[flag]}`}>
            !
          </span>
        )}
        {label}
      </span>
      <span
        className={[
          "text-[13px] wrap-break-word leading-relaxed",
          mono ? "font-mono" : "",
          !value ? "text-text-muted" : flag === "high" ? "text-severity-high" : flag === "medium" ? "text-severity-medium" : "text-text-primary",
        ].join(" ")}
      >
        {value ?? "(none)"}
      </span>
    </div>
  );
}
