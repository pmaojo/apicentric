import React, { useEffect, useState } from "react";
import {
  listServices,
  Service,
  startSimulator,
  stopSimulator,
  shareService,
  connectService,
  ConnectServiceRequest,
} from "../api/client";

const SimulatorControls: React.FC = () => {
  const [services, setServices] = useState<Service[]>([]);
  const [shareInfo, setShareInfo] = useState<{ peer: string; token: string } | null>(
    null
  );

  const [showShare, setShowShare] = useState(false);
  const [shareServiceForm, setShareServiceForm] = useState("");
  const [shareError, setShareError] = useState("");

  const [showConnect, setShowConnect] = useState(false);
  const [connectForm, setConnectForm] = useState<ConnectServiceRequest>({
    peer: "",
    service: "",
    port: 8080,
    token: "",
  });
  const [connectErrors, setConnectErrors] = useState<{
    peer?: string;
    service?: string;
    port?: string;
  }>({});

  const refresh = async () => {
    try {
      const list = await listServices();
      setServices(list);
    } catch (error) {
      console.error("Failed to fetch services:", error);
      // Optionally, set an error state to display in the UI
    }
  };

  const submitShare = async () => {
    if (!shareServiceForm.trim()) {
      setShareError("Service name is required");
      return;
    }
    try {
      const [peer, token] = await shareService(shareServiceForm.trim());
      setShareInfo({ peer, token });
      setShareServiceForm("");
      setShareError("");
      setShowShare(false);
    } catch (error) {
      console.error("Failed to share service:", error);
      setShareError((error as Error).message);
    }
  };

  const submitConnect = async () => {
    const errors: { peer?: string; service?: string; port?: string } = {};
    if (!connectForm.peer.trim()) errors.peer = "Peer ID is required";
    if (!connectForm.service.trim()) errors.service = "Service name is required";
    const portNum = Number(connectForm.port);
    if (!Number.isFinite(portNum) || portNum <= 0)
      errors.port = "Port must be a positive number";

    setConnectErrors(errors);
    if (Object.keys(errors).length > 0) return;

    try {
      await connectService({ ...connectForm, port: portNum });
      setConnectForm({ peer: "", service: "", port: 8080, token: "" });
      setShowConnect(false);
      setConnectErrors({});
    } catch (error) {
      console.error("Failed to connect to service:", error);
      // Optionally set a connection error state
    }
  };

  const handleStartSimulator = async (serviceId: string) => {
    try {
      await startSimulator(serviceId);
      await refresh();
    } catch (error) {
      console.error(`Failed to start simulator for ${serviceId}:`, error);
    }
  };

  const handleStopSimulator = async (serviceId: string) => {
    try {
      await stopSimulator(serviceId);
      await refresh();
    } catch (error) {
      console.error(`Failed to stop simulator for ${serviceId}:`, error);
    }
  };

  useEffect(() => {
    refresh();
  }, []);

  return (
    <div>
      {/* These controls are now per-service */}
      <button onClick={() => setShowShare(true)}>Share Service</button>
      <button onClick={() => setShowConnect(true)}>Connect Service</button>

      {showShare && (
        <div>
          <input
            placeholder="Service name to share"
            value={shareServiceForm}
            onChange={(e) => {
              setShareServiceForm(e.target.value);
              setShareError("");
            }}
          />
          {shareError && <div style={{ color: "red" }}>{shareError}</div>}
          <button onClick={submitShare}>Submit</button>
          <button
            onClick={() => {
              setShowShare(false);
              setShareServiceForm("");
              setShareError("");
            }}
          >
            Cancel
          </button>
        </div>
      )}

      {showConnect && (
        <div>
          <input
            placeholder="Peer ID"
            value={connectForm.peer}
            onChange={(e) =>
              setConnectForm({ ...connectForm, peer: e.target.value })
            }
          />
          {connectErrors.peer && (
            <div style={{ color: "red" }}>{connectErrors.peer}</div>
          )}
          <input
            placeholder="Service name"
            value={connectForm.service}
            onChange={(e) =>
              setConnectForm({ ...connectForm, service: e.target.value })
            }
          />
          {connectErrors.service && (
            <div style={{ color: "red" }}>{connectErrors.service}</div>
          )}
          <input
            type="number"
            placeholder="Local port"
            value={connectForm.port}
            onChange={(e) =>
              setConnectForm({ ...connectForm, port: Number(e.target.value) })
            }
          />
          {connectErrors.port && (
            <div style={{ color: "red" }}>{connectErrors.port}</div>
          )}
          <input
            placeholder="Token (optional)"
            value={connectForm.token}
            onChange={(e) =>
              setConnectForm({ ...connectForm, token: e.target.value })
            }
          />
          <button onClick={submitConnect}>Submit</button>
          <button
            onClick={() => {
              setShowConnect(false);
              setConnectForm({
                peer: "",
                service: "",
                port: 8080,
                token: "",
              });
              setConnectErrors({});
            }}
          >
            Cancel
          </button>
        </div>
      )}

      {shareInfo && (
        <pre>{`Peer: ${shareInfo.peer}\nToken: ${shareInfo.token}`}</pre>
      )}
      
      <h3>Services</h3>
      <table>
        <thead>
          <tr>
            <th>Name</th>
            <th>Status</th>
            <th>Port</th>
            <th>Actions</th>
          </tr>
        </thead>
        <tbody>
          {services.map((service) => (
            <tr key={service.id}>
              <td>{service.name}</td>
              <td>{service.is_running ? "Running" : "Stopped"}</td>
              <td>{service.port || "N/A"}</td>
              <td>
                {service.is_running ? (
                  <button onClick={() => handleStopSimulator(service.id)}>
                    Stop
                  </button>
                ) : (
                  <button onClick={() => handleStartSimulator(service.id)}>
                    Start
                  </button>
                )}
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
};

export default SimulatorControls;
