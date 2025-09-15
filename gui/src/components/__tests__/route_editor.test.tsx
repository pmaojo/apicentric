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

describe("RouteEditor", () => {
  beforeEach(() => {
    window.history.replaceState({}, "Test", "/route_editor?service=test.yaml");
  });

  test("loads service and displays endpoint", async () => {
    render(<RouteEditor />);
    expect(await screen.findByDisplayValue("GET")).toBeInTheDocument();
    expect(screen.getByDisplayValue("/hello")).toBeInTheDocument();
  });

  test("adds response and validates path", async () => {
    render(<RouteEditor />);
    await screen.findByDisplayValue("GET");

    fireEvent.click(screen.getByText("Add Response"));
    expect(screen.getAllByText(/Status:/i)).toHaveLength(1);

    fireEvent.change(screen.getByLabelText("Path:"), { target: { value: "" } });
    fireEvent.click(screen.getByText("Save"));
    expect(await screen.findByText("Path is required")).toBeInTheDocument();
  });
});

