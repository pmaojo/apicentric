import React, { useState } from "react";
import { useServicePort } from "../ports/ServicePort";
import { invoke } from "@tauri-apps/api/tauri";
import { load as loadYaml, dump as dumpYaml } from "js-yaml";
import Sidebar from "./Sidebar";

interface ServiceInfo {
  name: string;
  port: number;
  is_running: boolean;
}

interface ServicePort {
  load(path: string): Promise<any>;
  save(path: string, data: any): Promise<void>;
  list(): Promise<ServiceInfo[]>;
}


const ServiceManager: React.FC = () => {
  const [path, setPath] = useState("");
  const [name, setName] = useState("");
  const [port, setPort] = useState("");
  const [endpoints, setEndpoints] = useState<any[]>([]);
  const [types, setTypes] = useState("");

  const port = useServicePort();

  const load = async () => {
    const content = await port.loadService(path);
    setYaml(content);
  };

  const save = async () => {
    await port.saveService(path, yaml);

  };

  const exportTypes = async () => {
    const t = await port.exportTypes(path);
    setTypes(t);
  };

  const addEndpoint = () =>
    setEndpoints([
      ...endpoints,
      {
        method: "",
        path: "",
        headers: {},
        payload: "",
        responses: { strategy: "sequential", list: [] },
      },
    ]);

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
      <div>
        <label htmlFor="name">Name:</label>
        <input
          id="name"
          value={name}
          onChange={(e) => setName(e.target.value)}
        />
        {errors.name && (
          <span style={{ color: "red", marginLeft: 4 }}>{errors.name}</span>
        )}
      </div>
      <div>
        <label htmlFor="port">Port:</label>
        <input
          id="port"
          type="number"
          value={port}
          onChange={(e) => setPort(e.target.value)}
        />
        {errors.port && (
          <span style={{ color: "red", marginLeft: 4 }}>{errors.port}</span>
        )}
      </div>
      <Sidebar
        servicePath={path}
        endpoints={endpoints.map((e) => ({
          method: e.method,
          path: e.path,
        }))}
        onAdd={addEndpoint}
      />
      <pre>{types}</pre>
    </div>
  );
};

export default ServiceManager;
