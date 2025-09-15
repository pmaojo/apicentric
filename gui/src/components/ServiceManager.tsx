import React, { useState } from "react";
import { useServicePort } from "../ports/ServicePort";

const ServiceManager: React.FC = () => {
  const [path, setPath] = useState("");
  const [yaml, setYaml] = useState("");
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
