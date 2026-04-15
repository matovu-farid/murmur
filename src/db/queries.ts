import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";

export interface AppConfig {
  transcription: { mode: "local" | "api"; model: string; api_key: string };
  ai_cleanup: { enabled: boolean; custom_instructions: string };
  hotkey: { key: string; mode: "hold" | "toggle" };
  voice_commands: { enabled: boolean };
  general: { auto_start: boolean; context_aware: boolean };
}

export interface TranscriptionEntry {
  id: number;
  timestamp: string;
  raw_text: string;
  cleaned_text: string;
  duration_secs: number;
}

export function useConfig() {
  return useQuery<AppConfig>({
    queryKey: ["config"],
    queryFn: () => invoke<AppConfig>("get_config"),
  });
}

export function useUpdateConfig() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (config: AppConfig) => invoke("update_config", { config }),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["config"] });
    },
  });
}

export function useHistory() {
  return useQuery<TranscriptionEntry[]>({
    queryKey: ["history"],
    queryFn: () => invoke<TranscriptionEntry[]>("get_history"),
  });
}

export function useSearchHistory(query: string) {
  return useQuery<TranscriptionEntry[]>({
    queryKey: ["history", "search", query],
    queryFn: () => invoke<TranscriptionEntry[]>("search_history", { query }),
    enabled: query.length > 0,
  });
}

export function useDeleteHistoryEntry() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (id: number) => invoke("delete_history_entry", { id }),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["history"] });
    },
  });
}

export function useClearHistory() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: () => invoke("clear_history"),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["history"] });
    },
  });
}

export function useExportHistory() {
  return useMutation({ mutationFn: () => invoke<string>("export_history") });
}

export function useAccessibilityPermission() {
  return useQuery<boolean>({
    queryKey: ["accessibility"],
    queryFn: () => invoke<boolean>("check_accessibility_permission"),
  });
}
