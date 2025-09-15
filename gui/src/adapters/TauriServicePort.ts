import { invoke } from "@tauri-apps/api/tauri";
import { ServicePort, ServiceInfo, LogEntry } from "../ports/ServicePort";

export class TauriServicePort implements ServicePort {
  listServices(): Promise<ServiceInfo[]> {
    return invoke<ServiceInfo[]>("list_services");
  }

  startSimulator(): Promise<void> {
    return invoke("start_simulator");
  }

  stopSimulator(): Promise<void> {
    return invoke("stop_simulator");
  }

  shareService(service: string): Promise<[string, string]> {
    return invoke<[string, string]>("share_service", { service });
  }

  connectService(args: { service: string; peer: string; token: string; port: number }): Promise<void> {
    return invoke("connect_service", args);
  }

  loadService(path: string): Promise<string> {
    return invoke<string>("load_service", { path });
  }

  saveService(path: string, yaml: string): Promise<void> {
    return invoke("save_service", { path, yaml });
  }

  exportTypes(path: string): Promise<string> {
    return invoke<string>("export_types", { path });
  }

  getLogs(service: string, limit: number): Promise<LogEntry[]> {
    return invoke<LogEntry[]>("get_logs", { service, limit });
  }
}

export const tauriServicePort = new TauriServicePort();
