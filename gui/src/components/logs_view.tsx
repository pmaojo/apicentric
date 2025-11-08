import React, { useEffect, useState } from "react";
import { getServiceLogs } from "../api/client";

interface LogsViewProps {
  serviceId: string; // "global" for all, or a specific service ID
}

const LogsView: React.FC<LogsViewProps> = ({ serviceId }) => {
  const [logs, setLogs] = useState<string[]>([]);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const fetchLogs = async () => {
      try {
        setError(null);
        // The backend doesn't support global logs yet, so we'll just show a message.
        if (serviceId === "global") {
          setLogs(["Global log view is not yet implemented."]);
          return;
        }
        const fetchedLogs = await getServiceLogs(serviceId, 200);
        setLogs(fetchedLogs);
      } catch (err) {
        console.error(`Failed to fetch logs for ${serviceId}:`, err);
        setError((err as Error).message);
        setLogs([]);
      }
    };

    fetchLogs();

    // TODO: Implement real-time log streaming, perhaps with WebSockets
    const interval = setInterval(fetchLogs, 5000); // Poll every 5 seconds

    return () => clearInterval(interval);
  }, [serviceId]);

  return (
    <div style={{ background: "#222", color: "#eee", padding: "1rem", borderRadius: "4px", fontFamily: "monospace" }}>
      <h3>Logs for {serviceId}</h3>
      {error && <div style={{ color: "red" }}>Error: {error}</div>}
      <pre style={{ whiteSpace: "pre-wrap", wordBreak: "break-all", height: "300px", overflowY: "auto" }}>
        {logs.length > 0 ? logs.join("\n") : "No logs to display."}
      </pre>
    </div>
  );
};

export default LogsView;

