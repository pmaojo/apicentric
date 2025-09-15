import React from "react";
import { render, screen, fireEvent } from "@testing-library/react";
import RouteEditor from "../route_editor";
import { beforeEach, describe, expect, test, vi } from "vitest";
import "@testing-library/jest-dom";
import {
  ServicePort,
  ServicePortProvider,
} from "../../ports/ServicePort";

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

const mockPort: ServicePort = {
  listServices: vi.fn(),
  startSimulator: vi.fn(),
  stopSimulator: vi.fn(),
  shareService: vi.fn(),
  connectService: vi.fn(),
  loadService: vi.fn(),
  saveService: vi.fn(),
  exportTypes: vi.fn(),
  getLogs: vi.fn(),
};

beforeEach(() => {
  mockPort.loadService = vi.fn().mockResolvedValue(sampleYaml);
  mockPort.saveService = vi.fn().mockResolvedValue();
});

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
    render(
      <ServicePortProvider port={mockPort}>
        <RouteEditor />
      </ServicePortProvider>
    );
    expect(mockPort.loadService).toHaveBeenCalledWith("test.yaml");
    expect(await screen.findByDisplayValue("GET")).toBeInTheDocument();
    expect(screen.getByDisplayValue("/hello")).toBeInTheDocument();
  });


  test("adds response and validates path", async () => {
    render(
      <ServicePortProvider port={mockPort}>
        <RouteEditor />
      </ServicePortProvider>
    );
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

