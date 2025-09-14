import React, { useEffect, useState } from "react";
import { useNavigate } from "react-router-dom";
import { invoke } from "@tauri-apps/api/tauri";

export interface LogEntry {
  timestamp: string;
  method: string;
  path: string;
  status: number;
  service?: string;
  endpoint?: number;
}

interface ServiceInfo {
  name: string;
  port: number;
  is_running: boolean;
}

export const LogsView: React.FC = () => {
  const [logs, setLogs] = useState<LogEntry[]>([]);
  const [methodFilter, setMethodFilter] = useState("");
  const [pathFilter, setPathFilter] = useState("");
  const [statusFilter, setStatusFilter] = useState("");
  const navigate = useNavigate();

  useEffect(() => {
    let mounted = true;

    const fetchLogs = async () => {
      try {
        const services: ServiceInfo[] = await invoke("list_services");
        const running = services.filter((s) => s.is_running);
        const results = await Promise.all(
          running.map((s) =>
            invoke<LogEntry[]>("get_logs", { service: s.name, limit: 100 })
          )
        );
        if (mounted) {
          setLogs(results.flat());
        }
      } catch (e) {
        console.error("Failed to fetch logs", e);
      }
    };

    fetchLogs();
    const id = setInterval(fetchLogs, 2000);
    return () => {
      mounted = false;
      clearInterval(id);
    };
  }, []);

  const filtered = logs.filter((log) => {
    const methodMatch =
      !methodFilter || log.method.toLowerCase().includes(methodFilter.toLowerCase());
    const pathMatch =
      !pathFilter || log.path.toLowerCase().includes(pathFilter.toLowerCase());
    const statusMatch =
      !statusFilter || log.status.toString().includes(statusFilter);
    return methodMatch && pathMatch && statusMatch;
  });

  return (
    <div>
      <div>
        <input
          placeholder="method"
          value={methodFilter}
          onChange={(e) => setMethodFilter(e.target.value)}
        />
        <input
          placeholder="path"
          value={pathFilter}
          onChange={(e) => setPathFilter(e.target.value)}
        />
        <input
          placeholder="status"
          value={statusFilter}
          onChange={(e) => setStatusFilter(e.target.value)}
        />
      </div>
      <ul>
        {filtered.map((log, idx) => (
          <li key={idx}>
            <a
              href={`/route_editor?service=${log.service}&endpoint=${log.endpoint}`}
              onClick={(e) => {
                e.preventDefault();
                navigate(`/route_editor?service=${log.service}&endpoint=${log.endpoint}`);
              }}
            >
              [{log.method}] {log.path} - {log.status}
            </a>
          </li>
        ))}
      </ul>
    </div>
  );
};

export default LogsView;
