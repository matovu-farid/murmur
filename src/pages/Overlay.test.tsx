import { render, screen } from "@testing-library/react";
import { describe, it, expect, vi, beforeEach } from "vitest";

import * as tauriEvents from "../hooks/useTauriEvents";

vi.mock("../hooks/useTauriEvents", () => ({
  useDictationState: vi.fn(() => "idle"),
  useDictationError: vi.fn(() => null),
  useDictationResult: vi.fn(() => null),
}));

import { Overlay } from "./Overlay";

describe("Overlay", () => {
  beforeEach(() => {
    vi.mocked(tauriEvents.useDictationState).mockReturnValue("idle");
    vi.mocked(tauriEvents.useDictationError).mockReturnValue(null);
    vi.mocked(tauriEvents.useDictationResult).mockReturnValue(null);
  });

  it("returns null when idle with no error/result", () => {
    const { container } = render(<Overlay />);
    expect(container.innerHTML).toBe("");
  });

  it("shows recording state with waveform", () => {
    vi.mocked(tauriEvents.useDictationState).mockReturnValue("recording");
    const { container } = render(<Overlay />);
    expect(screen.getByText("Recording...")).toBeInTheDocument();
    // Waveform renders bars
    const bars = container.querySelectorAll(".w-1");
    expect(bars.length).toBeGreaterThan(0);
  });

  it("shows error message", () => {
    vi.mocked(tauriEvents.useDictationError).mockReturnValue("Mic not found");
    render(<Overlay />);
    expect(screen.getByText("Mic not found")).toBeInTheDocument();
  });

  it("shows spinner during transcribing", () => {
    vi.mocked(tauriEvents.useDictationState).mockReturnValue("transcribing");
    const { container } = render(<Overlay />);
    expect(screen.getByText("Transcribing...")).toBeInTheDocument();
    expect(container.querySelector(".animate-spin")).toBeInTheDocument();
  });
});
