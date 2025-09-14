import React, { useState } from "react";
import { invoke } from "@tauri-apps/api/tauri";

const ServiceManager: React.FC = () => {
  const [path, setPath] = useState("");
  const [yaml, setYaml] = useState("");
  const [types, setTypes] = useState("");

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
    <div>
      <div>
        <input
          placeholder="service.yaml"
          value={path}
          onChange={(e) => setPath(e.target.value)}
        />
        <button onClick={load}>Load</button>
        <button onClick={save}>Save</button>
        <button onClick={exportTypes}>Export TS Types</button>
      </div>
      <textarea
        style={{ width: "100%", height: "200px" }}
        value={yaml}
        onChange={(e) => setYaml(e.target.value)}
      />
      <pre>{types}</pre>
    </div>
  );
};

export default ServiceManager;
