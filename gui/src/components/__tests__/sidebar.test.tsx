import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import Sidebar from "../Sidebar";
import { describe, test, expect, vi, beforeEach } from "vitest";
import { invoke } from "@tauri-apps/api/tauri";
import { MemoryRouter } from "react-router-dom";
import "@testing-library/jest-dom";

const sampleYaml = `name: svc
endpoints:
  - method: GET
    path: /one
`;

vi.mock("@tauri-apps/api/tauri", () => ({
  invoke: vi.fn(),
}));

const invokeMock = invoke as unknown as vi.Mock;

describe("Sidebar", () => {
  beforeEach(() => {
    invokeMock.mockReset();
  });

  const renderSidebar = () =>
    render(
      <MemoryRouter>
        <Sidebar />
      </MemoryRouter>
    );

  test("adds service", async () => {
    invokeMock.mockImplementation((cmd: string) => {
      if (cmd === "list_services") return Promise.resolve([]);
      if (cmd === "save_service") return Promise.resolve();
      return Promise.resolve("");
    });
    vi.spyOn(window, "prompt").mockImplementation(() => "test.yaml");
    renderSidebar();
    await waitFor(() => expect(invokeMock).toHaveBeenCalledWith("list_services"));
    fireEvent.click(screen.getByRole("button", { name: "+" }));
    expect(await screen.findByText("test.yaml")).toBeInTheDocument();
  });

  test("deletes endpoint", async () => {
    invokeMock.mockImplementation((cmd: string, args: any) => {
      if (cmd === "list_services") return Promise.resolve([{ name: "svc.yaml" }]);
      if (cmd === "load_service") return Promise.resolve(sampleYaml);
      if (cmd === "save_service") return Promise.resolve();
      return Promise.resolve("");
    });
    renderSidebar();
    await screen.findByText("svc");
    fireEvent.click(screen.getAllByRole("button", { name: "+" })[1]);
    await screen.findByText("GET /one");
    fireEvent.click(screen.getAllByRole("button", { name: "X" })[1]);
    await waitFor(() =>
      expect(screen.queryByText("GET /one")).not.toBeInTheDocument()
    );
  });
});
