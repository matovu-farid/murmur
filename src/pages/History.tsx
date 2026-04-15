import { useState } from "react";
import {
  useHistory,
  useSearchHistory,
  useDeleteHistoryEntry,
  useClearHistory,
  useExportHistory,
} from "../db/queries";
import { HistoryEntryCard } from "../components/HistoryEntry";

export function History() {
  const [search, setSearch] = useState("");
  const historyQuery = useHistory();
  const searchQuery = useSearchHistory(search);
  const deleteEntry = useDeleteHistoryEntry();
  const clearHistory = useClearHistory();
  const exportHistory = useExportHistory();

  const entries = search.length > 0 ? searchQuery.data : historyQuery.data;
  const isLoading =
    search.length > 0 ? searchQuery.isLoading : historyQuery.isLoading;

  function handleExport() {
    exportHistory.mutate(undefined, {
      onSuccess: (json) => {
        const blob = new Blob([json], { type: "application/json" });
        const url = URL.createObjectURL(blob);
        const a = document.createElement("a");
        a.href = url;
        a.download = "wipr-history.json";
        a.click();
        URL.revokeObjectURL(url);
      },
    });
  }

  function handleClear() {
    if (confirm("Are you sure you want to clear all history?")) {
      clearHistory.mutate();
    }
  }

  return (
    <div className="min-h-screen bg-surface p-6 max-w-lg mx-auto">
      <div className="flex items-center justify-between mb-6">
        <h1 className="text-xl font-bold text-text">History</h1>
        <div className="flex gap-2">
          <button
            onClick={handleExport}
            disabled={exportHistory.isPending}
            className="px-3 py-1.5 text-xs font-medium rounded-lg bg-surface-light text-text hover:bg-primary/20 transition-colors disabled:opacity-50"
          >
            Export
          </button>
          <button
            onClick={handleClear}
            disabled={clearHistory.isPending}
            className="px-3 py-1.5 text-xs font-medium rounded-lg bg-surface-light text-danger hover:bg-danger/20 transition-colors disabled:opacity-50"
          >
            Clear All
          </button>
        </div>
      </div>

      <input
        type="text"
        value={search}
        onChange={(e) => setSearch(e.target.value)}
        placeholder="Search history..."
        className="w-full bg-surface-light text-text text-sm rounded-xl px-4 py-2.5 mb-4 border border-white/10 focus:outline-none focus:border-primary"
      />

      {isLoading && (
        <p className="text-text-muted text-sm text-center py-8">Loading...</p>
      )}

      {!isLoading && (!entries || entries.length === 0) && (
        <p className="text-text-muted text-sm text-center py-8">
          {search.length > 0
            ? "No results found."
            : "No transcription history yet."}
        </p>
      )}

      <div className="space-y-3">
        {entries?.map((entry) => (
          <HistoryEntryCard
            key={entry.id}
            entry={entry}
            onDelete={(id) => deleteEntry.mutate(id)}
          />
        ))}
      </div>
    </div>
  );
}
