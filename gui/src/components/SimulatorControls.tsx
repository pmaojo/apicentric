import React, { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/tauri";

interface ServiceInfo {
  name: string;
  [key: string]: any;
}

const SimulatorControls: React.FC = () => {
  const [services, setServices] = useState<ServiceInfo[]>([]);

  const refresh = async () => {
    const list = await invoke<ServiceInfo[]>("list_services");
    setServices(list);
  };

  useEffect(() => {
    refresh();
  }, []);

  return (
    <div>
      <button onClick={() => invoke("start_simulator").then(refresh)}>
        Start Simulator
      </button>
      <button onClick={() => invoke("stop_simulator").then(refresh)}>
        Stop Simulator
      </button>
      <pre>{JSON.stringify(services, null, 2)}</pre>
    </div>
  );
};

export default SimulatorControls;
