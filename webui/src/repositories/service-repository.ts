/**
 * Service Repository - Data Access Layer
 * 
 * Handles all HTTP API calls and data transformation.
 * This layer knows about the external API but not about business rules.
 */

import { Service, ValidationResult } from '@/lib/types';
import { apiClient } from '@/infrastructure/api-client';

export interface ServiceRepository {
  list(): Promise<Service[]>;
  get(name: string): Promise<Service>;
  create(yaml: string, filename?: string): Promise<Service>;
  update(name: string, yaml: string): Promise<Service>;
  delete(name: string): Promise<void>;
  start(name: string): Promise<void>;
  stop(name: string): Promise<void>;
  validate(yaml: string): Promise<ValidationResult>;
}

export class ApiServiceRepository implements ServiceRepository {
  async list(): Promise<Service[]> {
    const response = await apiClient.get<any[]>('/api/services');
    
    // Transform API response to domain entities
    return (response || []).map(this.transformApiServiceToService);
  }

  async get(name: string): Promise<Service> {
    const response = await apiClient.get<any>(
      `/api/services/${encodeURIComponent(name)}`
    );
    
    return this.transformApiServiceToService(response);
  }

  async create(yaml: string, filename?: string): Promise<Service> {
    const response = await apiClient.post<any>('/api/services', {
      yaml,
      filename,
    });
    
    return this.transformApiServiceToService(response);
  }

  async update(name: string, yaml: string): Promise<Service> {
    const response = await apiClient.put<any>(
      `/api/services/${encodeURIComponent(name)}`,
      { yaml }
    );
    
    return this.transformApiServiceToService(response);
  }

  async delete(name: string): Promise<void> {
    await apiClient.delete(`/api/services/${encodeURIComponent(name)}`);
  }

  async start(name: string): Promise<void> {
    await apiClient.post(`/api/services/${encodeURIComponent(name)}/start`);
  }

  async stop(name: string): Promise<void> {
    await apiClient.post(`/api/services/${encodeURIComponent(name)}/stop`);
  }

  async validate(yaml: string): Promise<ValidationResult> {
    try {
      const response = await apiClient.post<{ success: boolean; errors?: string[] }>('/api/services/validate', {
        yaml,
      });
      
      return {
        valid: response.success,
        errors: response.errors || [],
        warnings: []
      };
    } catch (error) {
      return {
        valid: false,
        errors: [error instanceof Error ? error.message : 'Validation failed'],
        warnings: []
      };
    }
  }

  // Transform API response to domain entity
  private transformApiServiceToService(apiService: any): Service {
    return {
      id: apiService.id || apiService.name || `service-${Date.now()}`,
      name: apiService.name || 'Unknown Service',
      status: (apiService.is_running ? 'running' : 'stopped') as 'running' | 'stopped',
      port: apiService.port || 0,
      version: apiService.version || '1.0.0',
      definition: apiService.yaml || apiService.definition || '',
      endpoints: (apiService.endpoints || []).map((ep: any) => ({
        path: ep.path || '/',
        method: ep.method || 'GET',
        status: ep.status || 200,
      })),
    };
  }
}