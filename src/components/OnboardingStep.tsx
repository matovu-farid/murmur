import { ReactNode } from "react";

export function OnboardingStep({
  step,
  totalSteps,
  title,
  children,
  onNext,
  onBack,
  nextLabel = "Next",
  nextDisabled = false,
}: {
  step: number;
  totalSteps: number;
  title: string;
  children: ReactNode;
  onNext: () => void;
  onBack?: () => void;
  nextLabel?: string;
  nextDisabled?: boolean;
}) {
  return (
    <div className="flex flex-col items-center justify-center min-h-screen p-8">
      <div className="w-full max-w-md">
        <div className="flex gap-1.5 mb-8">
          {Array.from({ length: totalSteps }).map((_, i) => (
            <div
              key={i}
              className={`h-1 flex-1 rounded-full ${i <= step ? "bg-primary" : "bg-surface-light"}`}
            />
          ))}
        </div>
        <h2 className="text-xl font-bold text-text mb-4">{title}</h2>
        <div className="mb-8">{children}</div>
        <div className="flex justify-between">
          {onBack ? (
            <button
              onClick={onBack}
              className="px-4 py-2 text-sm text-text-muted hover:text-text"
            >
              Back
            </button>
          ) : (
            <div />
          )}
          <button
            onClick={onNext}
            disabled={nextDisabled}
            className="px-6 py-2 text-sm font-medium rounded-xl bg-primary hover:bg-primary-hover text-white transition-colors disabled:opacity-50"
          >
            {nextLabel}
          </button>
        </div>
      </div>
    </div>
  );
}
