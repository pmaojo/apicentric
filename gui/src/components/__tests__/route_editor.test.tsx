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

    fireEvent.click(screen.getByText("Add Response"));
    expect(screen.getAllByText(/Status:/i)).toHaveLength(1);

    fireEvent.change(screen.getByLabelText("Path:"), { target: { value: "" } });
    fireEvent.click(screen.getByText("Save"));
    expect(mockPort.saveService).not.toHaveBeenCalled();
    expect(await screen.findByText("Path is required")).toBeInTheDocument();
  });
});

