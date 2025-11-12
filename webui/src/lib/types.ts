/**
 * @fileoverview Type definitions for the Apicentric application.
 */

/**
 * Represents a single API endpoint.
 */
export type Endpoint = {
  method: 'GET' | 'POST' | 'PUT' | 'DELETE' | 'PATCH';
  path: string;
  description: string;
};

/**
 * Represents a mock service in the UI.
 */
export type Service = {
  id: string;
  name: string;
  status: 'running' | 'stopped';
  port: number;
  version: string;
  definition: string;
  endpoints: Endpoint[];
};

/**
 * Represents a single log entry.
 */
export type Log = {
  id: string;
  timestamp: string;
  service: string;
  method: 'GET' | 'POST' | 'PUT' | 'DELETE';
  route: string;
  status: number;
  ip: string;
};

/**
 * Represents the active view in the main layout.
 */
export type View =
  | 'dashboard'
  | 'services'
  | 'recording'
  | 'ai-generator'
  | 'plugin-generator'
  | 'contract-testing'
  | 'code-generator'
  | 'logs';

/**
 * Represents a service as fetched from the API.
 */
export type ApiService = {
  id?: string; // Optional because it might not come from the Rust side
  name: string;
  version?: string;
  description?: string;
  definition?: string;
  port: number;
  server?: { port: number; base_path: string };
  base_path?: string;
  endpoints: {
    method: string;
    path: string;
    description?: string;
  }[];
  is_running: boolean;
  endpoints_count: number;
};

/**
 * Represents the overall status of the simulator.
 */
export type SimulatorStatus = {
    is_active: boolean;
    services_count: number;
    active_services: ApiService[];
};
