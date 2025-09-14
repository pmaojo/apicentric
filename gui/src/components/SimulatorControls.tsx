import React, { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/tauri";

interface ServiceInfo {
  name: string;
  [key: string]: any;
}

const SimulatorControls: React.FC = () => {
  const [services, setServices] = useState<ServiceInfo[]>([]);
  const [shareInfo, setShareInfo] = useState<{ peer: string; token: string } | null>(
    null
  );

  const refresh = async () => {
    const list = await invoke<ServiceInfo[]>("list_services");
    setServices(list);
  };

  const handleShare = async () => {
    const service = prompt("Service to share?");
    if (!service) return;
    const [peer, token] = await invoke<[string, string]>("share_service", {
      service,
    });
    setShareInfo({ peer, token });
  };

  const handleConnect = async () => {
    const peer = prompt("Peer ID?");
    if (!peer) return;
    const service = prompt("Service name?");
    if (!service) return;
    const portStr = prompt("Local port?", "8080");
    const port = Number(portStr || 0);
    const token = prompt("Token?") || "";
    await invoke("connect_service", { peer, token, service, port });
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
      <button onClick={handleShare}>Share Service</button>
      <button onClick={handleConnect}>Connect Service</button>
      {shareInfo && (
        <pre>{`Peer: ${shareInfo.peer}\nToken: ${shareInfo.token}`}</pre>
      )}
      <pre>{JSON.stringify(services, null, 2)}</pre>
    </div>
  );
};

export default SimulatorControls;
