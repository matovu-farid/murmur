import {
  useDictationState,
  useDictationError,
  useDictationResult,
} from "../hooks/useTauriEvents";
import { Waveform } from "../components/Waveform";

const stateLabels: Record<string, string> = {
  idle: "",
  recording: "Recording...",
  transcribing: "Transcribing...",
  cleaning: "Cleaning up...",
  processing: "Processing...",
  inserting: "Inserting...",
};

export function Overlay() {
  const state = useDictationState();
  const error = useDictationError();
  const result = useDictationResult();

  if (state === "idle" && !error && !result) return null;

  return (
    <div className="min-h-screen flex items-start justify-center pt-4 bg-transparent">
      <div className="bg-surface/95 backdrop-blur-md rounded-2xl px-6 py-4 shadow-2xl border border-white/10 flex items-center gap-4 min-w-64">
        {state === "recording" && <Waveform isActive={true} />}
        {state !== "idle" && (
          <span className="text-text text-sm font-medium">
            {stateLabels[state]}
          </span>
        )}
        {error && <span className="text-danger text-sm">{error}</span>}
        {result && state === "idle" && (
          <span className="text-success text-sm">Inserted</span>
        )}
        {(state === "transcribing" ||
          state === "cleaning" ||
          state === "processing") && (
          <div className="w-4 h-4 border-2 border-primary border-t-transparent rounded-full animate-spin" />
        )}
      </div>
    </div>
  );
}
