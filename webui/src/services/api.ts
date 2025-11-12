import type { ApiService, Service, SimulatorStatus } from '@/lib/types';

/**
 * @fileoverview API service functions for interacting with the backend.
 * This file assumes the apicentric cloud server is running on localhost:8080.
 */

const API_BASE_URL = 'http://localhost:8080/api/simulator';

/**
 * Fetches the list of all available services from the simulator backend.
 * This function is deprecated in favor of fetchSimulatorStatus.
 * @returns {Promise<ApiService[]>} A promise that resolves to an array of services.
 * @deprecated Use fetchSimulatorStatus instead to get a complete view of the simulator.
 */
export async function fetchServices(): Promise<ApiService[]> {
  const response = await fetch(`${API_BASE_URL}/status`);
  if (!response.ok) {
    throw new Error('Network response was not ok');
  }
  const result = await response.json();
  // Adapt the new status structure to the old ApiService[] format if needed, or update consumers
  return result.data?.active_services || [];
}


/**
 * Fetches the overall status of the simulator, including all service details.
 * @returns {Promise<SimulatorStatus>} A promise that resolves to the simulator's status.
 */
export async function fetchSimulatorStatus(): Promise<SimulatorStatus> {
    const response = await fetch(`${API_BASE_URL}/status`);
    if (!response.ok) {
        throw new Error('Failed to fetch simulator status');
    }
    const result = await response.json();
    return result.data;
}

/**
 * Starts the simulator.
 * @returns {Promise<any>} A promise that resolves when the simulator has been started.
 */
export async function startSimulator(): Promise<any> {
    const response = await fetch(`${API_BASE_URL}/start`, { method: 'POST' });
    if (!response.ok) {
        const errorData = await response.json().catch(() => ({ error: 'Failed to start simulator' }));
        throw new Error(errorData.error);
    }
    return response.json();
}

/**
 * Stops the simulator.
 * @returns {Promise<any>} A promise that resolves when the simulator has been stopped.
 */
export async function stopSimulator(): Promise<any> {
    const response = await fetch(`${API_BASE_URL}/stop`, { method: 'POST' });
    if (!response.ok) {
        const errorData = await response.json().catch(() => ({ error: 'Failed to stop simulator' }));
        throw new Error(errorData.error);
    }
    return response.json();
}

/**
 * Validates a service definition by sending it to the backend.
 * @param {string} definition - The YAML or JSON string of the service definition.
 * @returns {Promise<{ success: boolean, message?: string, error?: string }>} A promise that resolves to the validation result.
 */
export async function validateService(definition: string): Promise<{ success: boolean, message?: string, error?: string }> {
    const response = await fetch('/api/simulator/validate', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ definition }),
    });

    const result = await response.json();

    if (!response.ok) {
        throw new Error(result.error || 'Failed to validate service.');
    }
    
    return result;
}

/**
 * Creates a new GraphQL service by sending a request to the backend.
 * @param {string} name - The name of the new GraphQL service.
 * @param {number} port - The port number for the new service.
 * @returns {Promise<Service>} A promise that resolves to the newly created service.
 */
export async function createGraphQLService(name: string, port: number): Promise<Service> {
  const response = await fetch('/api/services/create-graphql', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ name, port }),
  });

  if (!response.ok) {
    const result = await response.json();
    throw new Error(result.error || 'Failed to create GraphQL service.');
  }

  return response.json();
}

/**
 * Runs contract tests for a given service.
 * @param {Service} service - The service to test.
 * @returns {Promise<any>} A promise that resolves with the test results.
 */
export async function runContractTests(service: Service): Promise<any> {
    const response = await fetch('/api/contract-testing', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(service),
    });

    if (!response.ok) {
        const errorData = await response.json().catch(() => ({ error: `Failed to run tests for ${service.name}` }));
        throw new Error(errorData.error);
    }

    return response.json();
}

/**
 * Generates client code for a given service definition.
 * @param {string} definition - The YAML or JSON string of the service definition.
 * @param {'typescript' | 'react-query' | 'react-components'} target - The code generation target.
 * @returns {Promise<string>} A promise that resolves to the generated code.
 */
export async function generateCode(definition: string, target: 'typescript' | 'react-query' | 'react-components'): Promise<string> {
    const response = await fetch('/api/codegen', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ definition, target }),
    });

    const result = await response.json();

    if (!response.ok) {
        throw new Error(result.error || `Failed to generate ${target}`);
    }

    return result.code;
}
