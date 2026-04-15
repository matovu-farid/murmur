import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { OnboardingStep } from "../components/OnboardingStep";
import {
  useAccessibilityPermission,
  useConfig,
  useUpdateConfig,
} from "../db/queries";
import {
  useDictationState,
  useDictationError,
} from "../hooks/useTauriEvents";
import { Waveform } from "../components/Waveform";

const TOTAL_STEPS = 6;

export function Onboarding() {
  const [step, setStep] = useState(0);
  const [mode, setMode] = useState<"local" | "api">("local");
  const [apiKey, setApiKey] = useState("");
  const { data: hasAccessibility, refetch: recheckAccessibility } =
    useAccessibilityPermission();
  const { data: config } = useConfig();
  const updateConfig = useUpdateConfig();
  const dictationState = useDictationState();
  const dictationError = useDictationError();

  function next() {
    if (step < TOTAL_STEPS - 1) setStep(step + 1);
  }
  function back() {
    if (step > 0) setStep(step - 1);
  }

  function saveTranscriptionMode() {
    if (!config) return;
    updateConfig.mutate({
      ...config,
      transcription: {
        ...config.transcription,
        mode,
        api_key: mode === "api" ? apiKey : config.transcription.api_key,
      },
    });
    next();
  }

  async function finish() {
    try {
      await getCurrentWindow().close();
    } catch {
      // fallback if window API unavailable
    }
  }

  // Step 0: Welcome
  if (step === 0) {
    return (
      <OnboardingStep
        step={0}
        totalSteps={TOTAL_STEPS}
        title="Welcome to Murmur"
        onNext={next}
        nextLabel="Get Started"
      >
        <p className="text-sm text-text-muted leading-relaxed">
          Murmur is a voice-to-text tool that lets you dictate text anywhere on
          your Mac. Press a hotkey, speak, and your words appear wherever your
          cursor is.
        </p>
      </OnboardingStep>
    );
  }

  // Step 1: Accessibility permission
  if (step === 1) {
    return (
      <OnboardingStep
        step={1}
        totalSteps={TOTAL_STEPS}
        title="Input Monitoring Permission"
        onNext={next}
        onBack={back}
        nextDisabled={!hasAccessibility}
      >
        <p className="text-sm text-text-muted mb-4">
          Murmur needs Input Monitoring permission to detect your hotkey and
          insert text into other applications.
        </p>
        <div className="flex items-center gap-3 mb-4">
          <div
            className={`w-3 h-3 rounded-full ${hasAccessibility ? "bg-success" : "bg-danger"}`}
          />
          <span className="text-sm text-text">
            {hasAccessibility ? "Permission granted" : "Permission required"}
          </span>
        </div>
        <div className="flex gap-2">
          <button
            onClick={() => invoke("open_accessibility_settings_cmd")}
            className="px-4 py-2 text-sm rounded-lg bg-surface-light text-text hover:bg-primary/20 transition-colors"
          >
            Open Settings
          </button>
          <button
            onClick={() => recheckAccessibility()}
            className="px-4 py-2 text-sm rounded-lg bg-surface-light text-text-muted hover:text-text transition-colors"
          >
            Check Again
          </button>
        </div>
        <div className="mt-4 p-3 bg-surface rounded-lg border border-white/10">
          <p className="text-xs text-text-muted">
            After granting permission, you may need to restart Murmur for the
            change to take effect.
          </p>
        </div>
      </OnboardingStep>
    );
  }

  // Step 2: Microphone permission info
  if (step === 2) {
    return (
      <OnboardingStep
        step={2}
        totalSteps={TOTAL_STEPS}
        title="Microphone Access"
        onNext={next}
        onBack={back}
      >
        <p className="text-sm text-text-muted leading-relaxed">
          When you first use the hotkey to record, macOS will automatically show
          a dialog asking for microphone permission. Please allow it so Murmur
          can capture your voice.
        </p>
        <div className="mt-4 p-3 bg-surface rounded-lg border border-white/10">
          <p className="text-xs text-text-muted">
            You can manage this later in System Settings &gt; Privacy &amp;
            Security &gt; Microphone.
          </p>
        </div>
      </OnboardingStep>
    );
  }

  // Step 3: Transcription mode
  if (step === 3) {
    return (
      <OnboardingStep
        step={3}
        totalSteps={TOTAL_STEPS}
        title="Transcription Mode"
        onNext={saveTranscriptionMode}
        onBack={back}
        nextDisabled={mode === "api" && apiKey.length === 0}
      >
        <p className="text-sm text-text-muted mb-4">
          Choose how your speech is transcribed.
        </p>
        <div className="space-y-3">
          <label
            className={`flex items-start gap-3 p-3 rounded-lg border cursor-pointer transition-colors ${mode === "local" ? "border-primary bg-primary/10" : "border-white/10 bg-surface"}`}
          >
            <input
              type="radio"
              name="mode"
              checked={mode === "local"}
              onChange={() => setMode("local")}
              className="mt-1"
            />
            <div>
              <span className="text-sm text-text font-medium">
                Local (Whisper)
              </span>
              <p className="text-xs text-text-muted mt-0.5">
                Runs on your Mac. Private, no internet needed. Requires model
                download.
              </p>
            </div>
          </label>
          <label
            className={`flex items-start gap-3 p-3 rounded-lg border cursor-pointer transition-colors ${mode === "api" ? "border-primary bg-primary/10" : "border-white/10 bg-surface"}`}
          >
            <input
              type="radio"
              name="mode"
              checked={mode === "api"}
              onChange={() => setMode("api")}
              className="mt-1"
            />
            <div>
              <span className="text-sm text-text font-medium">API</span>
              <p className="text-xs text-text-muted mt-0.5">
                Uses a cloud API. Faster, more accurate, requires API key.
              </p>
            </div>
          </label>
        </div>
        {mode === "api" && (
          <div className="mt-4">
            <label className="text-sm text-text block mb-1.5">API Key</label>
            <input
              type="password"
              value={apiKey}
              onChange={(e) => setApiKey(e.target.value)}
              placeholder="Enter your API key"
              className="w-full bg-surface text-text text-sm rounded-lg px-3 py-2 border border-white/10 focus:outline-none focus:border-primary"
            />
          </div>
        )}
      </OnboardingStep>
    );
  }

  // Step 4: Test recording
  if (step === 4) {
    return (
      <OnboardingStep
        step={4}
        totalSteps={TOTAL_STEPS}
        title="Test Recording"
        onNext={next}
        onBack={back}
        nextLabel="Continue"
      >
        <p className="text-sm text-text-muted mb-4">
          Try pressing your hotkey to test recording. You should see the
          waveform animation below when recording is active.
        </p>
        <div className="flex flex-col items-center gap-4 p-6 bg-surface rounded-xl border border-white/10">
          <Waveform isActive={dictationState === "recording"} />
          <span className="text-sm text-text-muted capitalize">
            {dictationState === "idle" ? "Press hotkey to start" : dictationState}
          </span>
          {dictationError && (
            <span className="text-xs text-danger">{dictationError}</span>
          )}
        </div>
      </OnboardingStep>
    );
  }

  // Step 5: Done
  return (
    <OnboardingStep
      step={5}
      totalSteps={TOTAL_STEPS}
      title="You're all set!"
      onNext={finish}
      onBack={back}
      nextLabel="Finish"
    >
      <p className="text-sm text-text-muted leading-relaxed">
        Murmur is ready to use. It will run in your menu bar. Press your hotkey
        anytime to start dictating. You can change settings from the tray menu.
      </p>
      <div className="mt-4 p-3 bg-surface rounded-lg border border-white/10 space-y-2">
        <p className="text-xs text-text-muted">
          <strong className="text-text">Tip:</strong> Right-click the menu bar
          icon to access settings, history, and more.
        </p>
      </div>
    </OnboardingStep>
  );
}
