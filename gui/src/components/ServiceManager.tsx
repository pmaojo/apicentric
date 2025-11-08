import React, { useState, useEffect } from "react";
import * as api from "../api/client";
import Sidebar from "./Sidebar";

interface ServiceInfo {
  name: string;
  port: number;
  is_running: boolean;
}

const ServiceManager: React.FC = () => {
  const [services, setServices] = useState<ServiceInfo[]>([]);
  const [selectedServicePath, setSelectedServicePath] = useState<string>("");
  const [yamlContent, setYamlContent] = useState<string>("");
  const [error, setError] = useState<string | null>(null);
  const [tsTypes, setTsTypes] = useState<string>("");

  useEffect(() => {
    const fetchServices = async () => {
      try {
        const serviceList = await api.listServices();
        setServices(serviceList);
      } catch (err: any) {
        setError(err.message);
      }
    };
    fetchServices();
  }, []);

  const handleLoad = async () => {
    if (!selectedServicePath) {
      setError("Please select a service path.");
      return;
    }
    try {
      setError(null);
      const content = await api.loadService(selectedServicePath);
      setYamlContent(content);
    } catch (err: any) {
      setError(err.message);
    }
  };

  const handleSave = async () => {
    if (!selectedServicePath) {
      setError("Please select a service path.");
      return;
    }
    try {
      setError(null);
      await api.saveService(selectedServicePath, yamlContent);
      alert("Service saved successfully!");
    } catch (err: any) {
      setError(err.message);
    }
  };

  // This function needs to be implemented in the backend first
  const handleExportTypes = async () => {
     alert("Exporting types is not yet implemented in the cloud backend.");
     // try {
     //   const types = await api.exportTypes(selectedServicePath);
     //   setTsTypes(types);
     // } catch (err: any) {
     //   setError(err.message);
     // }
  };

  return (
    <div>
      {error && <div style={{ color: "red", marginBottom: "10px" }}>Error: {error}</div>}
      <div>
        <input
          placeholder="path/to/service.yaml"
          value={selectedServicePath}
          onChange={(e) => setSelectedServicePath(e.target.value)}
        />
        <button onClick={handleLoad}>Load</button>
        <button onClick={handleSave}>Save</button>
        <button onClick={handleExportTypes}>Export TS Types</button>
      </div>
      
      <textarea
        value={yamlContent}
        onChange={(e) => setYamlContent(e.target.value)}
        rows={20}
        cols={80}
        placeholder="YAML content will be loaded here..."
        style={{ fontFamily: "monospace", marginTop: "10px" }}
      />

      <div style={{ marginTop: "20px" }}>
        <h3>Loaded Services</h3>
        <ul>
          {services.map((service) => (
            <li key={service.name}>
              {service.name} (Port: {service.port}) - {service.is_running ? "Running" : "Stopped"}
            </li>
          ))}
        </ul>
      </div>

      {tsTypes && (
        <div style={{ marginTop: "20px" }}>
          <h3>Generated TypeScript Types</h3>
          <pre>{tsTypes}</pre>
        </div>
      )}
    </div>
  );
};

export default ServiceManager;
