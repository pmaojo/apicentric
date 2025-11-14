/**
 * Clean Architecture for Apicentric WebUI
 * 
 * Layers (from outer to inner):
 * 1. UI Components (presentation) - Only handle rendering and user interaction
 * 2. Hooks/State Management - Handle component state and side effects  
 * 3. Services/Use Cases - Business logic and orchestration
 * 4. Data Access - API calls and data transformation
 * 5. Entities/Models - Pure data structures
 */

// ============================================================================
// Layer 5: Entities/Models (Core Domain)
// ============================================================================

export interface Service {
  id: string;
  name: string;
  status: 'running' | 'stopped';
  port: number;
  version: string;
  definition: string;
  endpoints: Endpoint[];
}

export interface SystemMetrics {
  active_services: number;
  total_requests: number;
  requests_per_minute: number;
  average_response_time: number;
  error_rate: number;
  memory_usage: number;
  cpu_usage: number;
  uptime: number;
}

export interface RequestLog {
  id: string;
  timestamp: string;
  service: string;
  method: string;
  path: string;
  status: number;
  duration_ms: number;
}

// ============================================================================
// Layer 4: Data Access (Infrastructure)
// ============================================================================

export interface ServiceRepository {
  list(): Promise<Service[]>;
  get(name: string): Promise<Service>;
  create(yaml: string): Promise<Service>;
  update(name: string, yaml: string): Promise<Service>;
  delete(name: string): Promise<void>;
  start(name: string): Promise<void>;
  stop(name: string): Promise<void>;
}

export interface MetricsRepository {
  getCurrent(): Promise<SystemMetrics>;
  subscribe(callback: (metrics: SystemMetrics) => void): () => void;
}

export interface LogRepository {
  query(filters: LogFilters): Promise<RequestLog[]>;
  clear(): Promise<void>;
  subscribe(callback: (log: RequestLog) => void): () => void;
}

// ============================================================================
// Layer 3: Services/Use Cases (Application)
// ============================================================================

export interface ServiceManager {
  listServices(): Promise<Service[]>;
  getService(name: string): Promise<Service>;
  createService(yaml: string): Promise<Service>;
  updateService(name: string, yaml: string): Promise<Service>;
  deleteService(name: string): Promise<void>;
  startService(name: string): Promise<void>;
  stopService(name: string): Promise<void>;
  validateServiceDefinition(yaml: string): Promise<ValidationResult>;
}

export interface SystemMonitor {
  getMetrics(): Promise<SystemMetrics>;
  subscribeToMetrics(callback: (metrics: SystemMetrics) => void): () => void;
}

export interface LogManager {
  getLogs(filters?: LogFilters): Promise<RequestLog[]>;
  clearLogs(): Promise<void>;
  subscribeToLogs(callback: (log: RequestLog) => void): () => void;
  exportLogs(format: 'json' | 'csv'): Promise<string>;
}

// ============================================================================
// Layer 2: State Management (Interface Adapters)
// ============================================================================

export interface ServiceStore {
  services: Service[];
  loading: boolean;
  error: string | null;
  
  // Actions
  loadServices(): Promise<void>;
  createService(yaml: string): Promise<void>;
  updateService(name: string, yaml: string): Promise<void>;
  deleteService(name: string): Promise<void>;
  startService(name: string): Promise<void>;
  stopService(name: string): Promise<void>;
}

export interface SystemStore {
  metrics: SystemMetrics;
  logs: RequestLog[];
  loading: boolean;
  
  // Actions  
  loadMetrics(): Promise<void>;
  loadLogs(filters?: LogFilters): Promise<void>;
  clearLogs(): Promise<void>;
}

// ============================================================================
// Layer 1: UI Components (Presentation)
// ============================================================================

// Components only receive props and emit events
// NO direct API calls, NO business logic, NO data transformation

export interface DashboardProps {
  services: Service[];
  metrics: SystemMetrics;
  onServiceStart: (name: string) => void;
  onServiceStop: (name: string) => void;
  onRefresh: () => void;
}

export interface ServiceListProps {
  services: Service[];
  loading: boolean;
  onServiceCreate: (yaml: string) => void;
  onServiceUpdate: (name: string, yaml: string) => void;
  onServiceDelete: (name: string) => void;
  onServiceStart: (name: string) => void;
  onServiceStop: (name: string) => void;
}

export interface LogViewerProps {
  logs: RequestLog[];
  loading: boolean;
  onFilter: (filters: LogFilters) => void;
  onClear: () => void;
  onExport: (format: 'json' | 'csv') => void;
}