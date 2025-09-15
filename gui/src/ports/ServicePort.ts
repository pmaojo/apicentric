import React, { createContext, useContext } from "react";

export interface ServiceInfo {
  name: string;
  port: number;
  is_running: boolean;
}

export interface LogEntry {
  timestamp: string;
  method: string;
  path: string;
  status: number;
  service: string;
  endpoint?: number;
}

export interface ServicePort {
  listServices(): Promise<ServiceInfo[]>;
  startSimulator(): Promise<void>;
  stopSimulator(): Promise<void>;
  shareService(service: string): Promise<[string, string]>;
  connectService(args: { service: string; peer: string; token: string; port: number }): Promise<void>;
  loadService(path: string): Promise<string>;
  saveService(path: string, yaml: string): Promise<void>;
  exportTypes(path: string): Promise<string>;
  getLogs(service: string, limit: number): Promise<LogEntry[]>;
}

const ServicePortContext = createContext<ServicePort | null>(null);

export const ServicePortProvider: React.FC<{ port: ServicePort; children: React.ReactNode }> = ({
  port,
  children,
}) => React.createElement(ServicePortContext.Provider, { value: port }, children);

export const useServicePort = (): ServicePort => {
  const ctx = useContext(ServicePortContext);
  if (!ctx) throw new Error("ServicePort not provided");
  return ctx;
};

