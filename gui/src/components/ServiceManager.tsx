import React, { useState } from "react";
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

const servicePort: ServicePort = {
  load: async (path) => {
    const yamlStr = await invoke<string>("load_service", { path });
    return loadYaml(yamlStr);
  },
  save: async (path, data) => {
    const yamlStr = dumpYaml(data);
    await invoke("save_service", { path, yaml: yamlStr });
  },
  list: async () => {
    return invoke<ServiceInfo[]>("list_services");
  },
};

const ServiceManager: React.FC = () => {
  const [path, setPath] = useState("");
  const [name, setName] = useState("");
  const [port, setPort] = useState("");
  const [endpoints, setEndpoints] = useState<any[]>([]);
  const [types, setTypes] = useState("");
  const [errors, setErrors] = useState<Record<string, string>>({});

  const load = async () => {
    const content = await servicePort.load(path);
    setName(content.name || "");
    setPort(content.port ? String(content.port) : "");
    setEndpoints(content.endpoints || []);
  };

  const validate = async () => {
    const err: Record<string, string> = {};
    if (!name.trim()) err.name = "Service name is required";
    if (!port.trim()) err.port = "Port is required";
    const portNum = Number(port);
    if (!err.port && (isNaN(portNum) || portNum <= 0)) {
      err.port = "Port must be a number";
    }
    if (!err.port) {
      const services = await servicePort.list();
      const dup = services.find(
        (s) => s.port === portNum && s.name !== name
      );
      if (dup) err.port = "Port already in use";
    }
    setErrors(err);
    return Object.keys(err).length === 0;
  };

  const save = async () => {
    if (!(await validate())) return;
    const data = { name, port: Number(port), endpoints };
    await servicePort.save(path, data);
  };

  const exportTypes = async () => {
    const t = await invoke<string>("export_types", { path });
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
