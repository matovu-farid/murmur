import { ReactNode } from "react";

export function SettingsSection({
  title,
  children,
}: {
  title: string;
  children: ReactNode;
}) {
  return (
    <div className="mb-6">
      <h2 className="text-lg font-semibold text-text mb-3">{title}</h2>
      <div className="bg-surface-light rounded-xl p-4 space-y-4">
        {children}
      </div>
    </div>
  );
}

export function Toggle({
  label,
  description,
  checked,
  onChange,
}: {
  label: string;
  description?: string;
  checked: boolean;
  onChange: (v: boolean) => void;
}) {
  return (
    <label className="flex items-center justify-between cursor-pointer">
      <div>
        <span className="text-sm text-text">{label}</span>
        {description && (
          <p className="text-xs text-text-muted mt-0.5">{description}</p>
        )}
      </div>
      <button
        role="switch"
        aria-checked={checked}
        onClick={() => onChange(!checked)}
        className={`relative w-11 h-6 rounded-full transition-colors ${checked ? "bg-primary" : "bg-gray-600"}`}
      >
        <span
          className={`absolute top-0.5 left-0.5 w-5 h-5 rounded-full bg-white transition-transform ${checked ? "translate-x-5" : ""}`}
        />
      </button>
    </label>
  );
}

export function Select({
  label,
  value,
  options,
  onChange,
}: {
  label: string;
  value: string;
  options: { value: string; label: string }[];
  onChange: (v: string) => void;
}) {
  return (
    <div className="flex items-center justify-between">
      <span className="text-sm text-text">{label}</span>
      <select
        value={value}
        onChange={(e) => onChange(e.target.value)}
        className="bg-surface text-text text-sm rounded-lg px-3 py-1.5 border border-white/10 focus:outline-none focus:border-primary"
      >
        {options.map((o) => (
          <option key={o.value} value={o.value}>
            {o.label}
          </option>
        ))}
      </select>
    </div>
  );
}

export function TextInput({
  label,
  value,
  onChange,
  type = "text",
  placeholder,
}: {
  label: string;
  value: string;
  onChange: (v: string) => void;
  type?: string;
  placeholder?: string;
}) {
  return (
    <div className="flex flex-col gap-1.5">
      <span className="text-sm text-text">{label}</span>
      <input
        type={type}
        value={value}
        onChange={(e) => onChange(e.target.value)}
        placeholder={placeholder}
        className="bg-surface text-text text-sm rounded-lg px-3 py-2 border border-white/10 focus:outline-none focus:border-primary"
      />
    </div>
  );
}
