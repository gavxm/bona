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

  return (
    <div
      className={clsx(
        "flex items-baseline gap-4 px-4 py-1.5 transition-all duration-150",
        isHighlighted && "border-l-3 border-l-highlight-border bg-highlight",
        !isHighlighted && "border-l-3 border-l-transparent"
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
