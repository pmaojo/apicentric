import React, { useEffect, useState } from "react";
import { useNavigate } from "react-router-dom";
import { invoke } from "@tauri-apps/api/tauri";
import { listen, UnlistenFn } from "@tauri-apps/api/event";
import { save } from "@tauri-apps/api/dialog";
import { writeTextFile } from "@tauri-apps/api/fs";

export interface LogEntry {
  timestamp: string;
  method: string;
  path: string;
  status: number;
  service: string;
  endpoint?: number;
}

interface ServiceInfo {
  name: string;
  port: number;
  is_running: boolean;
}

export const LogsView: React.FC = () => {
  const [logsByService, setLogsByService] = useState<Record<string, LogEntry[]>>({});
  const [serviceFilter, setServiceFilter] = useState("");
  const [methodFilter, setMethodFilter] = useState("");
  const [statusFilter, setStatusFilter] = useState("");
  const navigate = useNavigate();

  useEffect(() => {
    let mounted = true;
    let unlisten: UnlistenFn | undefined;

    const init = async () => {
      try {
        const services: ServiceInfo[] = await invoke("list_services");
        const running = services.filter((s) => s.is_running);
        const results = await Promise.all(
          running.map((s) => invoke<LogEntry[]>("get_logs", { service: s.name, limit: 100 }))
        );
        if (mounted) {
          const initial: Record<string, LogEntry[]> = {};
          running.forEach((s, idx) => {
            initial[s.name] = results[idx];
          });
          setLogsByService(initial);
        }

        unlisten = await listen<LogEntry>("log", (event) => {
          const entry = event.payload;
          setLogsByService((prev) => {
            const serviceLogs = prev[entry.service] ? [entry, ...prev[entry.service]] : [entry];
            return { ...prev, [entry.service]: serviceLogs };
          });
        });
      } catch (e) {
        console.error("Failed to initialize logs", e);
      }
    };

    init();

    return () => {
      mounted = false;
      if (unlisten) {
        unlisten();
      }
    };
  }, []);

  const allLogs = Object.values(logsByService).flat();
  const filtered = allLogs.filter((log) => {
    const serviceMatch =
      !serviceFilter || log.service.toLowerCase().includes(serviceFilter.toLowerCase());
    const methodMatch =
      !methodFilter || log.method.toLowerCase().includes(methodFilter.toLowerCase());
    const statusMatch =
      !statusFilter || log.status.toString().includes(statusFilter);
    return serviceMatch && methodMatch && statusMatch;
  });

  const exportLogs = async () => {
    const filePath = await save({ filters: [{ name: "JSON", extensions: ["json"] }] });
    if (filePath) {
      const data = JSON.stringify(allLogs, null, 2);
      await writeTextFile(filePath, data);
    }
  };

  return (
    <div>
      <div>
        <input
          placeholder="service"
          value={serviceFilter}
          onChange={(e) => setServiceFilter(e.target.value)}
        />
        <input
          placeholder="method"
          value={methodFilter}
          onChange={(e) => setMethodFilter(e.target.value)}
        />
        <input
          placeholder="status"
          value={statusFilter}
          onChange={(e) => setStatusFilter(e.target.value)}
        />
        <button onClick={exportLogs}>Export</button>
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
              [{log.service}] [{log.method}] {log.path} - {log.status}
            </a>
          </li>
        ))}
      </ul>
    </div>
  );
};

export default LogsView;

