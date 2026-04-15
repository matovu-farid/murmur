import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, it, expect, vi } from "vitest";
import { OnboardingStep } from "./OnboardingStep";

describe("OnboardingStep", () => {
  const defaultProps = {
    step: 0,
    totalSteps: 3,
    title: "Welcome",
    onNext: vi.fn(),
  };

  it("renders title and children", () => {
    render(
      <OnboardingStep {...defaultProps}>
        <p>Child content</p>
      </OnboardingStep>
    );
    expect(screen.getByText("Welcome")).toBeInTheDocument();
    expect(screen.getByText("Child content")).toBeInTheDocument();
  });

  it("shows progress indicators", () => {
    const { container } = render(
      <OnboardingStep {...defaultProps} totalSteps={4}>
        <p>Content</p>
      </OnboardingStep>
    );
    const indicators = container.querySelectorAll(".rounded-full.h-1");
    expect(indicators).toHaveLength(4);
  });

  it("back button calls onBack", async () => {
    const user = userEvent.setup();
    const onBack = vi.fn();
    render(
      <OnboardingStep {...defaultProps} onBack={onBack}>
        <p>Content</p>
      </OnboardingStep>
    );
    await user.click(screen.getByText("Back"));
    expect(onBack).toHaveBeenCalled();
  });

  it("next button calls onNext", async () => {
    const user = userEvent.setup();
    const onNext = vi.fn();
    render(
      <OnboardingStep {...defaultProps} onNext={onNext}>
        <p>Content</p>
      </OnboardingStep>
    );
    await user.click(screen.getByText("Next"));
    expect(onNext).toHaveBeenCalled();
  });

  it("next button disabled when nextDisabled", () => {
    render(
      <OnboardingStep {...defaultProps} nextDisabled={true}>
        <p>Content</p>
      </OnboardingStep>
    );
    expect(screen.getByText("Next")).toBeDisabled();
  });
});
