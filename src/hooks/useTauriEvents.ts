import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";

export type DictationState =
  | "idle"
  | "recording"
  | "transcribing"
  | "cleaning"
  | "processing"
  | "inserting";

export function useDictationState() {
  const [state, setState] = useState<DictationState>("idle");
  useEffect(() => {
    const unlisten = listen<string>("dictation-state", (e) =>
      setState(e.payload as DictationState)
    );
    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);
  return state;
}

export function useDictationError() {
  const [error, setError] = useState<string | null>(null);
  useEffect(() => {
    const unlisten = listen<string>("dictation-error", (e) => {
      setError(e.payload);
      setTimeout(() => setError(null), 5000);
    });
    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);
  return error;
}

export function useDictationResult() {
  const [result, setResult] = useState<string | null>(null);
  useEffect(() => {
    const unlisten = listen<string>("dictation-result", (e) => {
      setResult(e.payload);
      setTimeout(() => setResult(null), 3000);
    });
    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);
  return result;
}
