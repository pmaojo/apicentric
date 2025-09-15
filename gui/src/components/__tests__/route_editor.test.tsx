import React from "react";
import { render, screen, fireEvent } from "@testing-library/react";
import RouteEditor from "../route_editor";
import { beforeEach, describe, expect, test, vi } from "vitest";
import "@testing-library/jest-dom";

const sampleYaml = `
name: sample
endpoints:
  - method: GET
    path: /hello
    headers: {}
    payload: ""
    responses:
      strategy: sequential
      list: []
`;

vi.mock("@tauri-apps/api/tauri", () => ({
  invoke: (cmd: string) => {
    if (cmd === "load_service") {
      return Promise.resolve(sampleYaml);
    }
    if (cmd === "save_service") {
      return Promise.resolve();
    }
    return Promise.resolve();
  },
}));

vi.mock("@monaco-editor/react", () => ({
  default: (props: any) => {
    const { value, onChange } = props;
    return (
      <textarea
        data-testid="editor"
        value={value}
        onChange={(e) => onChange(e.target.value)}
      />
    );
  },
}));

describe("RouteEditor", () => {
  beforeEach(() => {
    window.history.replaceState({}, "Test", "/route_editor?service=test.yaml");
  });

  test("loads service and displays endpoint", async () => {
    render(<RouteEditor />);
    expect(await screen.findByDisplayValue("GET")).toBeInTheDocument();
    expect(screen.getByDisplayValue("/hello")).toBeInTheDocument();
  });

  test("adds scenario and validates path live", async () => {
    render(<RouteEditor />);
    await screen.findByDisplayValue("GET");

    fireEvent.click(screen.getByText("Add Scenario"));
    expect(screen.getByText(/Status:/i)).toBeInTheDocument();

    const pathInput = screen.getByLabelText("Path:");
    fireEvent.change(pathInput, { target: { value: "invalid" } });
    expect(await screen.findByText("Path must start with /"))
      .toBeInTheDocument();
  });

  test("switches between response scenarios", async () => {
    render(<RouteEditor />);
    await screen.findByDisplayValue("GET");

    fireEvent.click(screen.getByText("Add Scenario"));
    fireEvent.change(screen.getByLabelText("Status:"), {
      target: { value: "201" },
    });
    fireEvent.click(screen.getByText("Add Scenario"));
    fireEvent.change(screen.getByLabelText("Status:"), {
      target: { value: "202" },
    });
    fireEvent.click(screen.getByText("Scenario 1"));
    expect(screen.getByLabelText("Status:")).toHaveValue(201);
  });
});

