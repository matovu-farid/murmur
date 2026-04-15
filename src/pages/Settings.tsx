import { useEffect, useState } from "react";
import { useConfig, useUpdateConfig, type AppConfig } from "../db/queries";
import {
  SettingsSection,
  Toggle,
  Select,
  TextInput,
} from "../components/SettingsSection";

export function Settings() {
  const { data: config, isLoading } = useConfig();
  const updateConfig = useUpdateConfig();
  const [local, setLocal] = useState<AppConfig | null>(null);

  useEffect(() => {
    if (config && !local) setLocal(config);
  }, [config, local]);

  if (isLoading || !local) {
    return (
      <div className="min-h-screen bg-surface flex items-center justify-center">
        <span className="text-text-muted text-sm">Loading settings...</span>
      </div>
    );
  }

  function update(patch: Partial<AppConfig>) {
    const next = { ...local!, ...patch };
    setLocal(next);
    updateConfig.mutate(next);
  }

  return (
    <div className="min-h-screen bg-surface p-6 max-w-lg mx-auto">
      <h1 className="text-xl font-bold text-text mb-6">Settings</h1>

      <SettingsSection title="General">
        <Toggle
          label="Launch at login"
          description="Start Wipr automatically when you log in"
          checked={local.general.auto_start}
          onChange={(v) =>
            update({ general: { ...local.general, auto_start: v } })
          }
        />
        <Select
          label="Recording mode"
          value={local.hotkey.mode}
          options={[
            { value: "hold", label: "Hold to record" },
            { value: "toggle", label: "Toggle on/off" },
          ]}
          onChange={(v) =>
            update({
              hotkey: {
                ...local.hotkey,
                mode: v as "hold" | "toggle",
              },
            })
          }
        />
        <TextInput
          label="Hotkey"
          value={local.hotkey.key}
          onChange={(v) => update({ hotkey: { ...local.hotkey, key: v } })}
          placeholder="e.g. CmdOrCtrl+Shift+Space"
        />
      </SettingsSection>

      <SettingsSection title="Transcription">
        <Select
          label="Mode"
          value={local.transcription.mode}
          options={[
            { value: "local", label: "Local (Whisper)" },
            { value: "api", label: "API" },
          ]}
          onChange={(v) =>
            update({
              transcription: {
                ...local.transcription,
                mode: v as "local" | "api",
              },
            })
          }
        />
        {local.transcription.mode === "local" && (
          <Select
            label="Model"
            value={local.transcription.model}
            options={[
              { value: "tiny", label: "Tiny (fastest)" },
              { value: "base", label: "Base" },
              { value: "small", label: "Small" },
              { value: "medium", label: "Medium" },
              { value: "large", label: "Large (most accurate)" },
            ]}
            onChange={(v) =>
              update({
                transcription: { ...local.transcription, model: v },
              })
            }
          />
        )}
        {local.transcription.mode === "api" && (
          <TextInput
            label="API Key"
            value={local.transcription.api_key}
            onChange={(v) =>
              update({
                transcription: { ...local.transcription, api_key: v },
              })
            }
            type="password"
            placeholder="Enter your API key"
          />
        )}
      </SettingsSection>

      <SettingsSection title="AI Cleanup">
        <Toggle
          label="Enable AI cleanup"
          description="Automatically clean up transcribed text with AI"
          checked={local.ai_cleanup.enabled}
          onChange={(v) =>
            update({ ai_cleanup: { ...local.ai_cleanup, enabled: v } })
          }
        />
        <Toggle
          label="Context-aware"
          description="Use surrounding text context for better cleanup"
          checked={local.general.context_aware}
          onChange={(v) =>
            update({ general: { ...local.general, context_aware: v } })
          }
        />
        {local.ai_cleanup.enabled && (
          <TextInput
            label="Custom instructions"
            value={local.ai_cleanup.custom_instructions}
            onChange={(v) =>
              update({
                ai_cleanup: { ...local.ai_cleanup, custom_instructions: v },
              })
            }
            placeholder="e.g. Use formal tone, fix grammar..."
          />
        )}
      </SettingsSection>

      <SettingsSection title="Voice Commands">
        <Toggle
          label="Enable voice commands"
          description="Use voice commands like 'select all', 'new line', etc."
          checked={local.voice_commands.enabled}
          onChange={(v) => update({ voice_commands: { enabled: v } })}
        />
      </SettingsSection>

      {updateConfig.isPending && (
        <p className="text-xs text-text-muted text-center">Saving...</p>
      )}
    </div>
  );
}
