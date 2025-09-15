import React, { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/tauri";
import { useNavigate } from "react-router-dom";
import LogsView from "./logs_view";

interface ServiceInfo {
  name: string;
  port: number;
  is_running: boolean;
}

const Dashboard: React.FC = () => {
  const [services, setServices] = useState<ServiceInfo[]>([]);
  const [path, setPath] = useState("");
  const [yaml, setYaml] = useState("");
  const [types, setTypes] = useState("");
  const navigate = useNavigate();

  const refresh = async () => {
    const list = await invoke<ServiceInfo[]>("list_services");
    setServices(list);
  };

  const startSimulator = async () => {
    await invoke("start_simulator");
    refresh();
  };

  const stopSimulator = async () => {
    await invoke("stop_simulator");
    refresh();
  };

  const shareService = async (service: string) => {
    const [peer, token] = await invoke<[string, string]>("share_service", {
      service,
    });
    alert(`Peer: ${peer}\nToken: ${token}`);
  };

  const connectService = async (service: string) => {
    const peer = prompt("Peer ID?");
    if (!peer) return;
    const portStr = prompt("Local port?", "8080");
    const port = Number(portStr || 0);
    const token = prompt("Token?") || "";
    await invoke("connect_service", { peer, token, service, port });
  };

  const load = async () => {
    const content = await invoke<string>("load_service", { path });
    setYaml(content);
  };

  const save = async () => {
    await invoke("save_service", { path, yaml });
  };

  const exportTypes = async () => {
    const t = await invoke<string>("export_types", { path });
    setTypes(t);
  };

  useEffect(() => {
    refresh();
  }, []);

  return (
    <div style={{ padding: "1rem", fontFamily: "sans-serif" }}>
      <h1>Dashboard</h1>
      <div style={{ marginBottom: "1rem" }}>
        <button onClick={startSimulator}>Start Simulator</button>
        <button onClick={stopSimulator} style={{ marginLeft: "0.5rem" }}>
          Stop Simulator
        </button>
      </div>

      <section style={{ marginBottom: "1rem" }}>
        <h2>Services</h2>
        <table style={{ width: "100%", borderCollapse: "collapse" }}>
          <thead>
            <tr>
              <th style={{ textAlign: "left" }}>Name</th>
              <th>Status</th>
              <th>Actions</th>
              <th>Links</th>
            </tr>
          </thead>
          <tbody>
            {services.map((s) => (
              <tr key={s.name}>
                <td>{s.name}</td>
                <td>
                  <span
                    style={{
                      display: "inline-block",
                      width: 10,
                      height: 10,
                      borderRadius: "50%",
                      backgroundColor: s.is_running ? "green" : "red",
                      marginRight: 4,
                    }}
                  />
                  {s.is_running ? "Running" : "Stopped"}
                </td>
                <td>
                  <button onClick={() => shareService(s.name)}>Share</button>
                  <button
                    onClick={() => connectService(s.name)}
                    style={{ marginLeft: "0.5rem" }}
                  >
                    Connect
                  </button>
                </td>
                <td>
                  <a
                    href="#logs"
                    onClick={(e) => {
                      e.preventDefault();
                      document
                        .getElementById("logs")
                        ?.scrollIntoView({ behavior: "smooth" });
                    }}
                    style={{ marginRight: "0.5rem" }}
                  >
                    Logs
                  </a>
                  <a
                    href={`/route_editor?service=${s.name}`}
                    onClick={(e) => {
                      e.preventDefault();
                      navigate(`/route_editor?service=${s.name}`);
                    }}
                  >
                    Route Editor
                  </a>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </section>

      <section style={{ marginBottom: "1rem" }}>
        <h2>Service Manager</h2>
        <div style={{ marginBottom: "0.5rem" }}>
          <input
            placeholder="service.yaml"
            value={path}
            onChange={(e) => setPath(e.target.value)}
          />
          <button onClick={load} style={{ marginLeft: "0.5rem" }}>
            Load
          </button>
          <button onClick={save} style={{ marginLeft: "0.5rem" }}>
            Save
          </button>
          <button onClick={exportTypes} style={{ marginLeft: "0.5rem" }}>
            Export TS Types
          </button>
        </div>
        <textarea
          style={{ width: "100%", height: 200 }}
          value={yaml}
          onChange={(e) => setYaml(e.target.value)}
        />
        <pre>{types}</pre>
      </section>

      <section id="logs">
        <h2>Logs</h2>
        <LogsView />
      </section>
    </div>
  );
};

export default Dashboard;
