import type { TranscriptionEntry } from "../db/queries";

export function HistoryEntryCard({
  entry,
  onDelete,
}: {
  entry: TranscriptionEntry;
  onDelete: (id: number) => void;
}) {
  const date = new Date(entry.timestamp);
  return (
    <div className="bg-surface-light rounded-xl p-4 group">
      <div className="flex items-start justify-between mb-2">
        <div className="flex items-center gap-2 text-xs text-text-muted">
          <span>
            {date.toLocaleDateString([], { month: "short", day: "numeric" })}
          </span>
          <span>
            {date.toLocaleTimeString([], {
              hour: "2-digit",
              minute: "2-digit",
            })}
          </span>
          <span>{entry.duration_secs.toFixed(1)}s</span>
        </div>
        <button
          onClick={() => onDelete(entry.id)}
          className="opacity-0 group-hover:opacity-100 text-text-muted hover:text-danger transition-all text-xs px-2 py-1 rounded"
        >
          Delete
        </button>
      </div>
      <p className="text-sm text-text mb-1">{entry.cleaned_text}</p>
      {entry.raw_text !== entry.cleaned_text && (
        <p className="text-xs text-text-muted italic">
          Raw: {entry.raw_text}
        </p>
      )}
    </div>
  );
}
