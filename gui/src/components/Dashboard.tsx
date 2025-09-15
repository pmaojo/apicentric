import React, { useState } from "react";
import { invoke } from "@tauri-apps/api/tauri";
import LogsView from "./logs_view";

const Dashboard: React.FC = () => {
  const [path, setPath] = useState("");
  const [yaml, setYaml] = useState("");
  const [types, setTypes] = useState("");

  const startSimulator = async () => {
    await invoke("start_simulator");
  };

  const stopSimulator = async () => {
    await invoke("stop_simulator");
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
