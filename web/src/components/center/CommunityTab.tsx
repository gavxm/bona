import { useInvestigation } from "../../context/useInvestigation";
import { FieldRow } from "./FieldRow";

/** Compute a relative label anchored to `referenceIso` (the investigation
 *  timestamp) so the display stays stable for cached investigations. */
function formatRelativeDate(iso: string, referenceIso?: string): string {
  try {
    const dt = new Date(iso);
    const anchor = referenceIso ? new Date(referenceIso) : new Date();
    const days = Math.floor((anchor.getTime() - dt.getTime()) / (1000 * 60 * 60 * 24));

    const month = dt.toLocaleString("en", { month: "short", year: "numeric" });

    if (days >= 365) {
      const years = Math.floor(days / 365);
      return `${month} (${years} year${years !== 1 ? "s" : ""} ago)`;
    } else if (days >= 30) {
      const months = Math.floor(days / 30);
      return `${month} (${months} month${months !== 1 ? "s" : ""} ago)`;
    }
    return `${month} (${days} day${days !== 1 ? "s" : ""} ago)`;
  } catch {
    return iso;
  }
}

export function CommunityTab() {
  const { investigation } = useInvestigation();
  const community = investigation?.community;
  if (!community) {
    return <p className="px-4 py-6 text-sm text-text-muted">No community data available.</p>;
  }

  const authorUrl = community.author
    ? `https://huggingface.co/${community.author}`
    : null;

  return (
    <div className="py-2">
      <div className="flex items-baseline gap-4 px-4 py-1.5 border-l-3 border-l-transparent">
        <span className="text-text-secondary w-36 shrink-0 text-xs">author</span>
        {authorUrl ? (
          <a
            href={authorUrl}
            target="_blank"
            rel="noopener noreferrer"
            className="text-sm text-link hover:underline"
          >
            {community.author}
          </a>
        ) : (
          <span className="text-sm text-text-muted">(unknown)</span>
        )}
      </div>
      <FieldRow
        label="account created"
        field="author_created_at"
        value={community.author_created_at ? formatRelativeDate(community.author_created_at, investigation?.investigated_at) : null}
      />
      <FieldRow
        label="models published"
        field="author_model_count"
        value={community.author_model_count?.toLocaleString() ?? null}
        mono
      />
      <FieldRow
        label="discussions"
        field="discussion_count"
        value={
          community.discussion_count != null
            ? `${community.discussion_count} (${community.closed_discussion_count ?? 0} closed)`
            : null
        }
      />
    </div>
  );
}
