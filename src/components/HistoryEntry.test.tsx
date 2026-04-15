import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, it, expect, vi } from "vitest";
import { HistoryEntryCard } from "./HistoryEntry";
import type { TranscriptionEntry } from "../db/queries";

const baseEntry: TranscriptionEntry = {
  id: 42,
  timestamp: "2025-01-15T10:30:00Z",
  raw_text: "hello world",
  cleaned_text: "Hello, world.",
  duration_secs: 3.5,
};

describe("HistoryEntryCard", () => {
  it("renders entry with cleaned text", () => {
    render(<HistoryEntryCard entry={baseEntry} onDelete={() => {}} />);
    expect(screen.getByText("Hello, world.")).toBeInTheDocument();
  });

  it("shows raw text when different from cleaned", () => {
    render(<HistoryEntryCard entry={baseEntry} onDelete={() => {}} />);
    expect(screen.getByText("Raw: hello world")).toBeInTheDocument();
  });

  it("does not show raw text when same as cleaned", () => {
    const entry = { ...baseEntry, raw_text: "Hello, world.", cleaned_text: "Hello, world." };
    render(<HistoryEntryCard entry={entry} onDelete={() => {}} />);
    expect(screen.queryByText(/^Raw:/)).not.toBeInTheDocument();
  });

  it("calls onDelete with correct id", async () => {
    const user = userEvent.setup();
    const onDelete = vi.fn();
    render(<HistoryEntryCard entry={baseEntry} onDelete={onDelete} />);
    await user.click(screen.getByText("Delete"));
    expect(onDelete).toHaveBeenCalledWith(42);
  });
});
