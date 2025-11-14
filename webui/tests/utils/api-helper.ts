import { APIRequestContext } from '@playwright/test';

export interface SimulatorStatus {
  is_active: boolean;
  services_count: number;
  active_services: Array<{
    name: string;
    port: number;
    base_path: string;
    is_running: boolean;
    endpoints?: Array<{
      method: string;
      path: string;
    }>;
  }>;
}

export interface ApiService {
  name: string;
  path: string;
  status: 'stopped' | 'starting' | 'running' | 'stopping' | 'failed';
  port: number;
  endpoints: Array<{
    method: string;
    path: string;
    responses: any[];
  }>;
}

/**
 * Helper class for backend API interactions during testing
 */
export class ApiTestHelper {
  constructor(private baseUrl: string = 'http://localhost:8080') {}

  async getSimulatorStatus(): Promise<SimulatorStatus> {
    const response = await fetch(`${this.baseUrl}/status`);
    if (!response.ok) {
      throw new Error(`Failed to get simulator status: ${response.statusText}`);
    }
    const result = await response.json();
    return result.data;
  }

  async startSimulator(): Promise<any> {
    const response = await fetch(`${this.baseUrl}/start`, { method: 'POST' });
    if (!response.ok) {
      const errorData = await response.json().catch(() => ({ error: 'Failed to start simulator' }));
      throw new Error(errorData.error || response.statusText);
    }
    return response.json();
  }

  async stopSimulator(): Promise<any> {
    const response = await fetch(`${this.baseUrl}/stop`, { method: 'POST' });
    if (!response.ok) {
      const errorData = await response.json().catch(() => ({ error: 'Failed to stop simulator' }));
      throw new Error(errorData.error || response.statusText);
    }
    return response.json();
  }

  async listServices(): Promise<ApiService[]> {
    const response = await fetch(`${this.baseUrl}/api/services`);
    if (!response.ok) {
      throw new Error(`Failed to list services: ${response.statusText}`);
    }
    const result = await response.json();
    return result.data || [];
  }

  async createService(yaml: string, filename?: string): Promise<any> {
    const response = await fetch(`${this.baseUrl}/api/services`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ yaml, filename }),
    });
    if (!response.ok) {
      const errorData = await response.json().catch(() => ({ error: 'Failed to create service' }));
      throw new Error(errorData.error || response.statusText);
    }
    return response.json();
  }

  async deleteService(name: string): Promise<void> {
    const response = await fetch(`${this.baseUrl}/api/services/${encodeURIComponent(name)}`, {
      method: 'DELETE',
    });
    if (!response.ok) {
      throw new Error(`Failed to delete service: ${response.statusText}`);
    }
  }

  async startService(name: string): Promise<void> {
    const response = await fetch(`${this.baseUrl}/api/services/${encodeURIComponent(name)}/start`, {
      method: 'POST',
    });
    if (!response.ok) {
      throw new Error(`Failed to start service: ${response.statusText}`);
    }
  }

  async stopService(name: string): Promise<void> {
    const response = await fetch(`${this.baseUrl}/api/services/${encodeURIComponent(name)}/stop`, {
      method: 'POST',
    });
    if (!response.ok) {
      throw new Error(`Failed to stop service: ${response.statusText}`);
    }
  }

  async getServiceStatus(name: string): Promise<any> {
    const response = await fetch(`${this.baseUrl}/api/services/${encodeURIComponent(name)}/status`);
    if (!response.ok) {
      throw new Error(`Failed to get service status: ${response.statusText}`);
    }
    const result = await response.json();
    return result.data;
  }

  async queryLogs(filters: any = {}): Promise<any> {
    const params = new URLSearchParams();
    Object.entries(filters).forEach(([key, value]) => {
      if (value !== undefined && value !== null) {
        params.append(key, String(value));
      }
    });
    
    const url = `${this.baseUrl}/api/logs${params.toString() ? '?' + params.toString() : ''}`;
    const response = await fetch(url);
    if (!response.ok) {
      throw new Error(`Failed to query logs: ${response.statusText}`);
    }
    const result = await response.json();
    return result.data;
  }

  async clearLogs(): Promise<void> {
    const response = await fetch(`${this.baseUrl}/api/logs`, {
      method: 'DELETE',
    });
    if (!response.ok) {
      throw new Error(`Failed to clear logs: ${response.statusText}`);
    }
  }

  async waitForSimulatorState(isActive: boolean, timeoutMs: number = 10000): Promise<void> {
    const startTime = Date.now();
    while (Date.now() - startTime < timeoutMs) {
      const status = await this.getSimulatorStatus();
      if (status.is_active === isActive) {
        return;
      }
      await new Promise(resolve => setTimeout(resolve, 500));
    }
    throw new Error(`Timeout waiting for simulator state: ${isActive}`);
  }

  async waitForServiceState(serviceName: string, isRunning: boolean, timeoutMs: number = 10000): Promise<void> {
    const startTime = Date.now();
    while (Date.now() - startTime < timeoutMs) {
      const status = await this.getSimulatorStatus();
      const service = status.active_services.find(s => s.name === serviceName);
      if (service && service.is_running === isRunning) {
        return;
      }
      await new Promise(resolve => setTimeout(resolve, 500));
    }
    throw new Error(`Timeout waiting for service ${serviceName} state: ${isRunning}`);
  }
}