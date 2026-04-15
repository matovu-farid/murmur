import { describe, it, expect, vi, beforeEach } from "vitest";
import { renderHook, waitFor } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";
import { createElement } from "react";
import type { AppConfig } from "./queries";
import { useConfig, useUpdateConfig } from "./queries";

function createWrapper() {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false } },
  });
  return ({ children }: { children: React.ReactNode }) =>
    createElement(QueryClientProvider, { client: queryClient }, children);
}

describe("queries", () => {
  beforeEach(() => {
    vi.mocked(invoke).mockReset();
  });

  it("useConfig calls invoke with get_config", async () => {
    const mockConfig: AppConfig = {
      transcription: { mode: "local", model: "base", api_key: "" },
      ai_cleanup: { enabled: false, custom_instructions: "" },
      hotkey: { key: "space", mode: "hold" },
      voice_commands: { enabled: false },
      general: { auto_start: false, context_aware: false },
    };
    vi.mocked(invoke).mockResolvedValue(mockConfig);

    const { result } = renderHook(() => useConfig(), {
      wrapper: createWrapper(),
    });

    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(invoke).toHaveBeenCalledWith("get_config");
    expect(result.current.data).toEqual(mockConfig);
  });

  it("useUpdateConfig calls invoke with correct args", async () => {
    vi.mocked(invoke).mockResolvedValue(undefined);

    const config: AppConfig = {
      transcription: { mode: "api", model: "large", api_key: "key123" },
      ai_cleanup: { enabled: true, custom_instructions: "fix grammar" },
      hotkey: { key: "ctrl", mode: "toggle" },
      voice_commands: { enabled: true },
      general: { auto_start: true, context_aware: true },
    };

    const { result } = renderHook(() => useUpdateConfig(), {
      wrapper: createWrapper(),
    });

    result.current.mutate(config);

    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(invoke).toHaveBeenCalledWith("update_config", { config });
  });

  it("AppConfig type has expected structure", () => {
    const config: AppConfig = {
      transcription: { mode: "local", model: "base", api_key: "" },
      ai_cleanup: { enabled: false, custom_instructions: "" },
      hotkey: { key: "space", mode: "hold" },
      voice_commands: { enabled: false },
      general: { auto_start: false, context_aware: false },
    };
    expect(config.transcription.mode).toBe("local");
    expect(config.ai_cleanup).toHaveProperty("enabled");
    expect(config.hotkey).toHaveProperty("key");
    expect(config.voice_commands).toHaveProperty("enabled");
    expect(config.general).toHaveProperty("auto_start");
  });
});
