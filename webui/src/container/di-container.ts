/**
 * Dependency Injection Container
 * 
 * Creates and manages all application dependencies.
 * This is where we wire up the clean architecture layers.
 */

import { ApiServiceRepository, ServiceRepository } from '@/repositories/service-repository';
import { ServiceManager, ServiceManagerInterface } from '@/services/service-manager';
import { webSocketManager, WebSocketManagerInterface } from '@/infrastructure/websocket-manager';

/**
 * Container for all application dependencies
 */
class DIContainer {
  private static instance: DIContainer;
  private _serviceRepository: ServiceRepository | null = null;
  private _serviceManager: ServiceManagerInterface | null = null;
  private _webSocketManager: WebSocketManagerInterface | null = null;

  private constructor() {}

  static getInstance(): DIContainer {
    if (!DIContainer.instance) {
      DIContainer.instance = new DIContainer();
    }
    return DIContainer.instance;
  }

  // Infrastructure Layer
  get webSocketManager(): WebSocketManagerInterface {
    if (!this._webSocketManager) {
      this._webSocketManager = webSocketManager;
    }
    return this._webSocketManager;
  }

  // Data Access Layer  
  get serviceRepository(): ServiceRepository {
    if (!this._serviceRepository) {
      this._serviceRepository = new ApiServiceRepository();
    }
    return this._serviceRepository;
  }

  // Business Logic Layer
  get serviceManager(): ServiceManagerInterface {
    if (!this._serviceManager) {
      this._serviceManager = new ServiceManager(
        this.serviceRepository,
        this.webSocketManager
      );
    }
    return this._serviceManager;
  }

  // For testing - allows injection of mocks
  setServiceRepository(repository: ServiceRepository): void {
    this._serviceRepository = repository;
    this._serviceManager = null; // Reset dependent services
  }

  setWebSocketManager(manager: WebSocketManagerInterface): void {
    this._webSocketManager = manager;
    this._serviceManager = null; // Reset dependent services
  }
}

// Export singleton instance
export const container = DIContainer.getInstance();

// Export individual services for convenience
export const serviceManager = container.serviceManager;
export const serviceRepository = container.serviceRepository;