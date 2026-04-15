import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, it, expect, vi } from "vitest";
import { Toggle, Select, TextInput } from "./SettingsSection";

describe("Toggle", () => {
  it("renders label and description", () => {
    render(
      <Toggle
        label="Test Label"
        description="Test Description"
        checked={false}
        onChange={() => {}}
      />
    );
    expect(screen.getByText("Test Label")).toBeInTheDocument();
    expect(screen.getByText("Test Description")).toBeInTheDocument();
  });

  it("calls onChange when clicked", async () => {
    const user = userEvent.setup();
    const onChange = vi.fn();
    render(
      <Toggle label="Toggle" checked={false} onChange={onChange} />
    );
    await user.click(screen.getByRole("switch"));
    expect(onChange).toHaveBeenCalledWith(true);
  });
});

describe("Select", () => {
  it("renders options", () => {
    const options = [
      { value: "a", label: "Option A" },
      { value: "b", label: "Option B" },
    ];
    render(
      <Select label="Pick" value="a" options={options} onChange={() => {}} />
    );
    expect(screen.getByText("Option A")).toBeInTheDocument();
    expect(screen.getByText("Option B")).toBeInTheDocument();
  });
});

describe("TextInput", () => {
  it("renders with placeholder", () => {
    render(
      <TextInput
        label="Name"
        value=""
        onChange={() => {}}
        placeholder="Enter name"
      />
    );
    expect(screen.getByPlaceholderText("Enter name")).toBeInTheDocument();
  });
});
