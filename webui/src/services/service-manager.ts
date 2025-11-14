/**
 * Service Manager - Use Case Layer
 * 
 * Orchestrates business logic without knowing about UI or API implementation details.
 * This is where business rules and validation logic lives.
 */

import { Service, ValidationResult } from '@/lib/types';
import { ServiceRepository } from '@/repositories/service-repository';
import { WebSocketManagerInterface } from '@/infrastructure/websocket-manager';

export interface ServiceManagerInterface {
  listServices(): Promise<Service[]>;
  getService(name: string): Promise<Service>;
  createService(yaml: string, filename?: string): Promise<Service>;
  updateService(name: string, yaml: string): Promise<Service>;
  deleteService(name: string): Promise<void>;
  startService(name: string): Promise<Service>;
  stopService(name: string): Promise<Service>;
  validateServiceDefinition(yaml: string): Promise<ValidationResult>;
  subscribeToServiceUpdates(callback: (service: Service) => void): () => void;
}

export class ServiceManager implements ServiceManagerInterface {
  constructor(
    private serviceRepository: ServiceRepository,
    private webSocketManager: WebSocketManagerInterface
  ) {}

  async listServices(): Promise<Service[]> {
    try {
      return await this.serviceRepository.list();
    } catch (error) {
      throw new Error(`Failed to list services: ${error instanceof Error ? error.message : 'Unknown error'}`);
    }
  }

  async getService(name: string): Promise<Service> {
    this.validateServiceName(name);
    
    try {
      return await this.serviceRepository.get(name);
    } catch (error) {
      throw new Error(`Failed to get service "${name}": ${error instanceof Error ? error.message : 'Unknown error'}`);
    }
  }

  async createService(yaml: string, filename?: string): Promise<Service> {
    // Business rule: Validate YAML before creating
    const validationResult = await this.validateServiceDefinition(yaml);
    if (!validationResult.isValid) {
      throw new Error(`Invalid service definition: ${validationResult.errors.join(', ')}`);
    }

    try {
      const service = await this.serviceRepository.create(yaml, filename);
      
      // Business rule: Log service creation
      console.log(`Service "${service.name}" created successfully`);
      
      return service;
    } catch (error) {
      throw new Error(`Failed to create service: ${error instanceof Error ? error.message : 'Unknown error'}`);
    }
  }

  async updateService(name: string, yaml: string): Promise<Service> {
    this.validateServiceName(name);
    
    // Business rule: Validate YAML before updating
    const validationResult = await this.validateServiceDefinition(yaml);
    if (!validationResult.isValid) {
      throw new Error(`Invalid service definition: ${validationResult.errors.join(', ')}`);
    }

    try {
      const service = await this.serviceRepository.update(name, yaml);
      
      // Business rule: Log service update
      console.log(`Service "${name}" updated successfully`);
      
      return service;
    } catch (error) {
      throw new Error(`Failed to update service "${name}": ${error instanceof Error ? error.message : 'Unknown error'}`);
    }
  }

  async deleteService(name: string): Promise<void> {
    this.validateServiceName(name);
    
    try {
      // Business rule: Stop service before deleting if it's running
      const service = await this.serviceRepository.get(name);
      if (service.status === 'running') {
        await this.stopService(name);
      }
      
      await this.serviceRepository.delete(name);
      
      // Business rule: Log service deletion
      console.log(`Service "${name}" deleted successfully`);
      
    } catch (error) {
      throw new Error(`Failed to delete service "${name}": ${error instanceof Error ? error.message : 'Unknown error'}`);
    }
  }

  async startService(name: string): Promise<Service> {
    this.validateServiceName(name);
    
    try {
      // Business rule: Check if service exists before starting
      const existingService = await this.serviceRepository.get(name);
      
      const service = await this.serviceRepository.startService(name);
      
      // Business rule: Log service start
      console.log(`Service "${name}" started successfully`);
      
      return service;
    } catch (error) {
      throw new Error(`Failed to start service "${name}": ${error instanceof Error ? error.message : 'Unknown error'}`);
    }
  }

  async stopService(name: string): Promise<Service> {
    this.validateServiceName(name);
    
    try {
      const service = await this.serviceRepository.stopService(name);
      
      // Business rule: Log service stop
      console.log(`Service "${name}" stopped successfully`);
      
      return service;
    } catch (error) {
      throw new Error(`Failed to stop service "${name}": ${error instanceof Error ? error.message : 'Unknown error'}`);
    }
  }

  async validateServiceDefinition(yaml: string): Promise<ValidationResult> {
    if (!yaml || yaml.trim().length === 0) {
      return {
        valid: false,
        errors: ['Service definition cannot be empty'],
        warnings: []
      };
    }

    try {
      return await this.serviceRepository.validate(yaml);
    } catch (error) {
      return {
        valid: false,
        errors: [error instanceof Error ? error.message : 'Validation failed'],
        warnings: []
      };
    }
  }

  subscribeToServiceUpdates(callback: (service: Service) => void): () => void {
    return this.webSocketManager.subscribe('service_status', (data: any) => {
      // Transform WebSocket data to Service entity
      const service: Partial<Service> = {
        name: data.service_name,
        status: data.status,
        port: data.port,
      };
      
      callback(service as Service);
    });
  }

  // Private business rules and validation
  private validateServiceName(name: string): void {
    if (!name || name.trim().length === 0) {
      throw new Error('Service name cannot be empty');
    }
    
    if (name.length > 100) {
      throw new Error('Service name cannot exceed 100 characters');
    }
    
    if (!/^[a-zA-Z0-9\-_]+$/.test(name)) {
      throw new Error('Service name can only contain letters, numbers, hyphens, and underscores');
    }
  }
}