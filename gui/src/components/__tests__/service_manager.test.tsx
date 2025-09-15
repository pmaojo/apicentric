import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { describe, test, expect, vi, beforeEach } from "vitest";
import "@testing-library/jest-dom";
import { load as loadYaml } from "js-yaml";

vi.mock("@tauri-apps/api/tauri", () => ({ invoke: vi.fn() }));
import { invoke } from "@tauri-apps/api/tauri";
import ServiceManager from "../ServiceManager";

describe("ServiceManager", () => {
  beforeEach(() => {
    invoke.mockReset();
  });

  test("saves service via port", async () => {
    invoke.mockImplementation((cmd: string) => {
      if (cmd === "list_services") return Promise.resolve([]);
      if (cmd === "save_service") return Promise.resolve();
      return Promise.resolve("");
    });

    render(<ServiceManager />);

    fireEvent.change(screen.getByPlaceholderText("service.yaml"), {
      target: { value: "svc.yaml" },
    });
    fireEvent.change(screen.getByLabelText("Name:"), {
      target: { value: "svc" },
    });
    fireEvent.change(screen.getByLabelText("Port:"), {
      target: { value: "8080" },
    });

    fireEvent.click(screen.getByText("Save"));

    await waitFor(() =>
      expect(invoke).toHaveBeenCalledWith(
        "save_service",
        expect.objectContaining({ path: "svc.yaml", yaml: expect.any(String) })
      )
    );
    const call = invoke.mock.calls.find(([c]) => c === "save_service");
    const args = call && call[1];
    const saved = loadYaml(args.yaml) as any;
    expect(saved.name).toBe("svc");
    expect(saved.port).toBe(8080);
  });

  test("shows validation errors", async () => {
    invoke.mockImplementation((cmd: string) => {
      if (cmd === "list_services")
        return Promise.resolve([{ name: "other", port: 8080, is_running: false }]);
      return Promise.resolve("");
    });

    render(<ServiceManager />);

    fireEvent.change(screen.getByPlaceholderText("service.yaml"), {
      target: { value: "svc.yaml" },
    });

    fireEvent.click(screen.getByText("Save"));

    expect(await screen.findByText("Service name is required")).toBeInTheDocument();
    expect(screen.getByText("Port is required")).toBeInTheDocument();

    fireEvent.change(screen.getByLabelText("Name:"), {
      target: { value: "svc" },
    });
    fireEvent.change(screen.getByLabelText("Port:"), {
      target: { value: "8080" },
    });

    fireEvent.click(screen.getByText("Save"));

    expect(await screen.findByText("Port already in use")).toBeInTheDocument();
  });
});

