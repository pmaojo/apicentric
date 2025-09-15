import React, { useEffect, useState } from "react";
import { useServicePort, ServiceInfo } from "../ports/ServicePort";

const SimulatorControls: React.FC = () => {
  const [services, setServices] = useState<ServiceInfo[]>([]);
  const [shareInfo, setShareInfo] = useState<{ peer: string; token: string } | null>(
    null
  );

  const [showShare, setShowShare] = useState(false);
  const [shareService, setShareService] = useState("");
  const [shareError, setShareError] = useState("");

  const [showConnect, setShowConnect] = useState(false);
  const [connectForm, setConnectForm] = useState({
    peer: "",
    service: "",
    port: "8080",
    token: "",
  });
  const [connectErrors, setConnectErrors] = useState<{
    peer?: string;
    service?: string;
    port?: string;
  }>({});

  const port = useServicePort();

  const refresh = async () => {
    const list = await port.listServices();
    setServices(list);
  };

  const submitShare = async () => {
    if (!shareService.trim()) {
      setShareError("Service name is required");
      return;
    }
    const [peer, token] = await port.shareService(shareService.trim());
    setShareInfo({ peer, token });
    setShareService("");
    setShareError("");
    setShowShare(false);
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

    await port.connectService({
      peer: connectForm.peer.trim(),
      service: connectForm.service.trim(),
      port: portNum,
      token: connectForm.token,
    });
    setConnectForm({ peer: "", service: "", port: "8080", token: "" });
    setShowConnect(false);
    setConnectErrors({});
  };

  useEffect(() => {
    refresh();
  }, []);

  return (
    <div>
      <button onClick={() => port.startSimulator().then(refresh)}>
        Start Simulator
      </button>
      <button onClick={() => port.stopSimulator().then(refresh)}>
        Stop Simulator
      </button>
      <button onClick={() => setShowShare(true)}>Share Service</button>
      <button onClick={() => setShowConnect(true)}>Connect Service</button>

      {showShare && (
        <div>
          <input
            placeholder="Service name"
            value={shareService}
            onChange={(e) => {
              setShareService(e.target.value);
              setShareError("");
            }}
          />
          {shareError && <div style={{ color: "red" }}>{shareError}</div>}
          <button onClick={submitShare}>Submit</button>
          <button
            onClick={() => {
              setShowShare(false);
              setShareService("");
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
            placeholder="Local port"
            value={connectForm.port}
            onChange={(e) =>
              setConnectForm({ ...connectForm, port: e.target.value })
            }
          />
          {connectErrors.port && (
            <div style={{ color: "red" }}>{connectErrors.port}</div>
          )}
          <input
            placeholder="Token"
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
                port: "8080",
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
      <pre>{JSON.stringify(services, null, 2)}</pre>
    </div>
  );
};

export default SimulatorControls;
