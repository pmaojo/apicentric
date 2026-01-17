/**
 * @fileoverview Type definitions for the Apicentric application.
 */

/**
 * Represents a single API endpoint.
 */
export type Endpoint = {
  method: 'GET' | 'POST' | 'PUT' | 'DELETE' | 'PATCH';
  path: string;
  description?: string;
  operationId?: string;
};

/**
 * Validation result for service definitions
 */
export interface ValidationResult {
  valid: boolean;
  errors: string[];
  warnings: string[];
}

/**
 * Log filters for querying request logs
 */
export interface LogFilters {
  search: string;
  service: string;
  method: string;
  status: string;
}

/**
 * Payload for creating a new service
 */
export interface CreateServicePayload {
  yaml: string;
  filename?: string;
}

/**
 * Payload for updating an existing service
 */
export interface UpdateServicePayload {
  yaml: string;
}

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
 * Represents a request log entry from the backend.
 */
export type RequestLogEntry = {
  timestamp: string;
  service: string;
  method: string;
  path: string;
  status: number;
  duration_ms?: number;
  request_headers?: Record<string, string>;
  response_headers?: Record<string, string>;
  request_body?: string;
  response_body?: string;
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
  | 'logs'
  | 'configuration'
  | 'marketplace'
  | 'iot';

/**
 * Represents an IoT Twin configuration.
 */
export interface TwinConfig {
  twin: {
    name: string;
    physics: Array<{
      variable: string;
      strategy: string;
      params: Record<string, any>;
    }>;
    transports: Array<{
      type: string;
      [key: string]: any;
    }>;
  };
}

/**
 * Represents a marketplace item.
 */
export interface MarketplaceItem {
  id: string;
  name: string;
  description: string;
  category: string;
  logo_url?: string;
  definition_url: string;
}

/**
 * Payload for importing from URL.
 */
export interface ImportUrlPayload {
  url: string;
  format?: string;
}

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
