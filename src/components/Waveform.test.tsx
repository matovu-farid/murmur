import { render, screen } from "@testing-library/react";
import { describe, it, expect } from "vitest";
import { Waveform } from "./Waveform";

describe("Waveform", () => {
  it("renders 12 bars", () => {
    const { container } = render(<Waveform isActive={false} />);
    const bars = container.querySelectorAll(".w-1");
    expect(bars).toHaveLength(12);
  });

  it("bars have correct CSS classes", () => {
    const { container } = render(<Waveform isActive={false} />);
    const bar = container.querySelector(".w-1");
    expect(bar).toHaveClass("bg-primary", "rounded-full", "transition-all", "duration-100");
  });
});
