import { useEffect, useRef } from "react";
import clsx from "clsx";
import { useInvestigation } from "../../context/useInvestigation";

interface FieldRowProps {
  label: string;
  field: string;
  value: string | null | undefined;
  mono?: boolean;
}

export function FieldRow({ label, field, value, mono = false }: FieldRowProps) {
  const { highlightedFields } = useInvestigation();
  const isHighlighted = highlightedFields.includes(field);
  const ref = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (isHighlighted && ref.current) {
      ref.current.scrollIntoView({ behavior: "smooth", block: "nearest" });
    }
  }, [isHighlighted]);

  return (
    <div
      ref={ref}
      className={clsx(
        "flex items-baseline gap-4 px-4 py-1.5 border-l-3 transition-all duration-300 ease-in-out",
        isHighlighted
          ? "border-l-highlight-border bg-[rgba(210,153,34,0.12)]"
          : "border-l-transparent bg-transparent"
      )}
    >
      <span className="text-text-secondary w-36 shrink-0 text-xs">
        {label}
      </span>
      <span
        className={clsx(
          "text-sm text-text-primary",
          mono && "font-mono",
          !value && "text-text-muted"
        )}
      >
        {value ?? "(none)"}
      </span>
    </div>
  );
}
