import { describe, it, expect } from "vitest";
import { renderHook } from "@testing-library/react";
import { useDictationState, useDictationError } from "./useTauriEvents";

describe("useTauriEvents", () => {
  it("useDictationState returns idle initially", () => {
    const { result } = renderHook(() => useDictationState());
    expect(result.current).toBe("idle");
  });

  it("useDictationError returns null initially", () => {
    const { result } = renderHook(() => useDictationError());
    expect(result.current).toBeNull();
  });
});
