import clsx from "clsx";
import type { SourceStatus } from "../../types";

export function StatusDot({ status }: { status: SourceStatus }) {
  const color =
    status.status === "ok"
      ? "bg-status-ok"
      : status.status === "failed"
        ? "bg-status-failed"
        : "bg-text-muted";

  return (
    <span
      className={clsx("inline-block w-2 h-2 rounded-full", color)}
      title={
        status.status === "ok"
          ? `${status.fetched_ms}ms`
          : status.status === "failed"
            ? status.reason
            : "not implemented"
      }
    />
  );
}
