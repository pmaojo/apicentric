import React from "react";

export interface LogEntry {
  timestamp: string;
  method: string;
  path: string;
  status: number;
  service?: string;
  endpoint?: number;
}

interface LogsViewProps {
  logs: LogEntry[];
}

export const LogsView: React.FC<LogsViewProps> = ({ logs }) => {
  const navigate = (service?: string, endpoint?: number) => {
    if (service !== undefined && endpoint !== undefined) {
      window.location.href = `route_editor.tsx?service=${service}&endpoint=${endpoint}`;
    }
  };

  return (
    <ul>
      {logs.map((log, idx) => (
        <li key={idx}>
          <a
            href={`route_editor.tsx?service=${log.service}&endpoint=${log.endpoint}`}
            onClick={(e) => {
              e.preventDefault();
              navigate(log.service, log.endpoint);
            }}
          >
            [{log.method}] {log.path} - {log.status}
          </a>
        </li>
      ))}
    </ul>
  );
};

export default LogsView;
