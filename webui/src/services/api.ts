import type { ApiService, Service, SimulatorStatus } from '@/lib/types';

/**
 * @fileoverview API service functions for interacting with the backend.
 * This file provides a comprehensive API client with JWT authentication,
 * automatic token refresh, and error handling.
 */

// Configuration
const DEFAULT_API_URL = process.env.NEXT_PUBLIC_API_URL || 'http://localhost:8080';
const STORAGE_KEY_API_URL = 'apicentric_api_url';
const TOKEN_STORAGE_KEY = 'apicentric_token';
const TOKEN_REFRESH_THRESHOLD = 5 * 60 * 1000; // 5 minutes before expiry

/**
 * Gets the configured API base URL.
 * Checks localStorage first for runtime overrides, then falls back to environment variable or default.
 */
export function getApiUrl(): string {
  if (typeof window !== 'undefined') {
    const storedUrl = localStorage.getItem(STORAGE_KEY_API_URL);
    if (storedUrl) {
      return storedUrl;
    }
  }
  return DEFAULT_API_URL;
}

/**
 * Sets a new API base URL at runtime.
 */
export function setApiUrl(url: string): void {
  if (typeof window !== 'undefined') {
    localStorage.setItem(STORAGE_KEY_API_URL, url);
  }
}

/**
 * Resets the API URL to the default configuration.
 */
export function resetApiUrl(): void {
  if (typeof window !== 'undefined') {
    localStorage.removeItem(STORAGE_KEY_API_URL);
  }
}

// Token management
let currentToken: string | null = null;
let tokenExpiryTime: number | null = null;

/**
 * Stores the JWT token securely.
 * Uses localStorage for now - can be upgraded to httpOnly cookies for production.
 */
function setToken(token: string): void {
  currentToken = token;
  if (typeof window !== 'undefined') {
    localStorage.setItem(TOKEN_STORAGE_KEY, token);
  }
  
  // Decode token to get expiry time
  try {
    const payload = JSON.parse(atob(token.split('.')[1]));
    tokenExpiryTime = payload.exp * 1000; // Convert to milliseconds
  } catch (e) {
    console.error('Failed to decode token:', e);
  }
}

/**
 * Retrieves the stored JWT token.
 */
function getToken(): string | null {
  if (currentToken) {
    return currentToken;
  }
  
  if (typeof window !== 'undefined') {
    currentToken = localStorage.getItem(TOKEN_STORAGE_KEY);
    return currentToken;
  }
  
  return null;
}

/**
 * Removes the stored JWT token.
 */
function clearToken(): void {
  currentToken = null;
  tokenExpiryTime = null;
  if (typeof window !== 'undefined') {
    localStorage.removeItem(TOKEN_STORAGE_KEY);
  }
}

/**
 * Checks if the token needs to be refreshed.
 */
function shouldRefreshToken(): boolean {
  if (!tokenExpiryTime) {
    return false;
  }
  
  const timeUntilExpiry = tokenExpiryTime - Date.now();
  return timeUntilExpiry < TOKEN_REFRESH_THRESHOLD && timeUntilExpiry > 0;
}

/**
 * Refreshes the JWT token.
 */
async function refreshToken(): Promise<void> {
  const token = getToken();
  if (!token) {
    throw new Error('No token to refresh');
  }
  
  const response = await fetch(`${getApiUrl()}/api/auth/refresh`, {
    method: 'POST',
    headers: {
      'Authorization': `Bearer ${token}`,
    },
  });
  
  if (!response.ok) {
    clearToken();
    throw new Error('Failed to refresh token');
  }
  
  const data = await response.json();
  setToken(data.token);
}

/**
 * Makes an authenticated API request with automatic token refresh.
 */
async function apiRequest<T>(
  endpoint: string,
  options: RequestInit = {}
): Promise<T> {
  // Check if token needs refresh
  if (shouldRefreshToken()) {
    try {
      await refreshToken();
    } catch (e) {
      console.error('Token refresh failed:', e);
    }
  }
  
  // Add authentication header if token exists
  const token = getToken();
  const headers: Record<string, string> = {
    'Content-Type': 'application/json',
    ...(options.headers as Record<string, string>),
  };
  
  if (token) {
    headers['Authorization'] = `Bearer ${token}`;
  }
  
  const response = await fetch(`${getApiUrl()}${endpoint}`, {
    ...options,
    headers,
  });
  
  // Handle 401 - try to refresh token once
  if (response.status === 401 && token) {
    try {
      await refreshToken();
      
      // Retry the request with new token
      const newToken = getToken();
      if (newToken) {
        const retryHeaders: Record<string, string> = {
          ...headers,
          'Authorization': `Bearer ${newToken}`,
        };
        const retryResponse = await fetch(`${getApiUrl()}${endpoint}`, {
          ...options,
          headers: retryHeaders,
        });
        
        if (!retryResponse.ok) {
          throw new Error(`API request failed: ${retryResponse.statusText}`);
        }
        
        return retryResponse.json();
      }
    } catch (e) {
      clearToken();
      throw new Error('Authentication failed');
    }
  }
  
  if (!response.ok) {
    const errorData = await response.json().catch(() => ({
      error: response.statusText,
    }));
    throw new Error(errorData.error || `Request failed: ${response.statusText}`);
  }
  
  return response.json();
}

// ============================================================================
// Authentication API
// ============================================================================

export interface RegisterRequest {
  username: string;
  password: string;
}

export interface LoginRequest {
  username: string;
  password: string;
}

export interface AuthResponse {
  token: string;
}

/**
 * Registers a new user.
 */
export async function register(username: string, password: string): Promise<AuthResponse> {
  const response = await apiRequest<AuthResponse>('/api/auth/register', {
    method: 'POST',
    body: JSON.stringify({ username, password }),
  });
  
  setToken(response.token);
  return response;
}

/**
 * Logs in a user.
 */
export async function login(username: string, password: string): Promise<AuthResponse> {
  const response = await apiRequest<AuthResponse>('/api/auth/login', {
    method: 'POST',
    body: JSON.stringify({ username, password }),
  });
  
  setToken(response.token);
  return response;
}

/**
 * Logs out the current user.
 */
export async function logout(): Promise<void> {
  try {
    await apiRequest('/api/auth/logout', {
      method: 'POST',
    });
  } finally {
    clearToken();
  }
}

/**
 * Gets the current user information.
 */
export async function getCurrentUser(): Promise<{ username: string }> {
  return apiRequest('/api/auth/me', {
    method: 'GET',
  });
}

/**
 * Checks if the user is authenticated.
 */
export function isAuthenticated(): boolean {
  return getToken() !== null;
}

// ============================================================================
// Service Management API
// ============================================================================

export interface ServiceResponse {
  name: string;
  path: string;
  status: 'stopped' | 'starting' | 'running' | 'stopping' | 'failed';
  port: number;
  endpoints: EndpointResponse[];
  uptime_seconds?: number;
}

export interface EndpointResponse {
  method: string;
  path: string;
  responses: any[];
}

export interface CreateServiceRequest {
  yaml: string;
  filename?: string;
}

export interface UpdateServiceRequest {
  yaml: string;
}

/**
 * Lists all services.
 */
export async function listServices(): Promise<ApiService[]> {
  const response = await apiRequest<{ success: boolean; data: any[] }>('/api/services');
  return response.data || [];
}

/**
 * Gets a specific service by name.
 */
export async function getService(name: string): Promise<ServiceResponse> {
  const response = await apiRequest<{ success: boolean; data: ServiceResponse }>(
    `/api/services/${encodeURIComponent(name)}`
  );
  return response.data;
}

/**
 * Creates a new service.
 */
export async function createService(yaml: string, filename?: string): Promise<ServiceResponse> {
  const response = await apiRequest<{ success: boolean; data: ServiceResponse }>('/api/services', {
    method: 'POST',
    body: JSON.stringify({ yaml, filename }),
  });
  return response.data;
}

/**
 * Updates an existing service.
 */
export async function updateService(name: string, yaml: string): Promise<ServiceResponse> {
  const response = await apiRequest<{ success: boolean; data: ServiceResponse }>(
    `/api/services/${encodeURIComponent(name)}`,
    {
      method: 'PUT',
      body: JSON.stringify({ yaml }),
    }
  );
  return response.data;
}

/**
 * Deletes a service.
 */
export async function deleteService(name: string): Promise<void> {
  await apiRequest(`/api/services/${encodeURIComponent(name)}`, {
    method: 'DELETE',
  });
}

/**
 * Starts a service.
 */
export async function startService(name: string): Promise<void> {
  await apiRequest(`/api/services/${encodeURIComponent(name)}/start`, {
    method: 'POST',
  });
}

/**
 * Stops a service.
 */
export async function stopService(name: string): Promise<void> {
  await apiRequest(`/api/services/${encodeURIComponent(name)}/stop`, {
    method: 'POST',
  });
}

/**
 * Gets the status of a service.
 */
export async function getServiceStatus(name: string): Promise<any> {
  const response = await apiRequest<{ success: boolean; data: any }>(
    `/api/services/${encodeURIComponent(name)}/status`
  );
  return response.data;
}

/**
 * Reloads all services.
 */
export async function reloadServices(): Promise<void> {
  await apiRequest('/api/services/reload', {
    method: 'POST',
  });
}

// ============================================================================
// Request Logs API
// ============================================================================

export interface RequestLogEntry {
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
}

export interface LogsQuery {
  limit?: number;
  service?: string;
  method?: string;
  status?: number;
  from?: string;
  to?: string;
}

export interface LogsResponse {
  logs: RequestLogEntry[];
  total: number;
  filtered: number;
}

/**
 * Queries request logs with optional filters.
 */
export async function queryLogs(query: LogsQuery = {}): Promise<LogsResponse> {
  const params = new URLSearchParams();
  if (query.limit) params.append('limit', query.limit.toString());
  if (query.service) params.append('service', query.service);
  if (query.method) params.append('method', query.method);
  if (query.status) params.append('status', query.status.toString());
  if (query.from) params.append('from', query.from);
  if (query.to) params.append('to', query.to);
  
  const response = await apiRequest<{ success: boolean; data: LogsResponse }>(
    `/api/logs?${params.toString()}`
  );
  return response.data;
}

/**
 * Clears all request logs.
 */
export async function clearLogs(): Promise<void> {
  await apiRequest('/api/logs', {
    method: 'DELETE',
  });
}

/**
 * Exports logs in the specified format.
 */
export async function exportLogs(format: 'json' | 'csv' = 'json'): Promise<string> {
  const response = await apiRequest<{ success: boolean; data: string }>(
    `/api/logs/export?format=${format}`
  );
  return response.data;
}

// ============================================================================
// Recording API
// ============================================================================

export interface StartRecordingRequest {
  target_url: string;
  proxy_port?: number;
}

export interface RecordingResponse {
  session_id: string;
  proxy_url: string;
  proxy_port: number;
  target_url: string;
  captured_count: number;
}

export interface RecordingStatusResponse {
  is_active: boolean;
  session_id?: string;
  proxy_url?: string;
  proxy_port?: number;
  target_url?: string;
  captured_count: number;
}

export interface CapturedRequest {
  method: string;
  path: string;
  headers: Record<string, string>;
  body?: string;
  response_status: number;
  response_headers: Record<string, string>;
  response_body?: string;
}

/**
 * Starts recording mode.
 */
export async function startRecording(targetUrl: string, proxyPort?: number): Promise<RecordingResponse> {
  const response = await apiRequest<{ success: boolean; data: RecordingResponse }>(
    '/api/recording/start',
    {
      method: 'POST',
      body: JSON.stringify({ target_url: targetUrl, proxy_port: proxyPort }),
    }
  );
  return response.data;
}

/**
 * Stops recording mode.
 */
export async function stopRecording(): Promise<{ captured_requests: CapturedRequest[] }> {
  const response = await apiRequest<{ success: boolean; data: { captured_requests: CapturedRequest[] } }>(
    '/api/recording/stop',
    {
      method: 'POST',
    }
  );
  return response.data;
}

/**
 * Gets the current recording status.
 */
export async function getRecordingStatus(): Promise<RecordingStatusResponse> {
  const response = await apiRequest<{ success: boolean; data: RecordingStatusResponse }>(
    '/api/recording/status'
  );
  return response.data;
}

/**
 * Generates a service definition from recorded requests.
 */
export async function generateServiceFromRecording(serviceName: string): Promise<ServiceResponse> {
  const response = await apiRequest<{ success: boolean; data: ServiceResponse }>(
    '/api/recording/generate',
    {
      method: 'POST',
      body: JSON.stringify({ service_name: serviceName }),
    }
  );
  return response.data;
}

// ============================================================================
// AI Generation API
// ============================================================================

export interface AiGenerateRequest {
  prompt: string;
  provider?: 'openai' | 'gemini' | 'local';
}

export interface AiGenerateResponse {
  yaml: string;
  validation_errors: string[];
}

export interface AiConfigResponse {
  is_configured: boolean;
  provider: string;
  model?: string;
  issues: string[];
}

/**
 * Generates a service definition using AI.
 */
export async function aiGenerate(prompt: string, provider?: string): Promise<AiGenerateResponse> {
  const response = await apiRequest<{ success: boolean; data: AiGenerateResponse }>(
    '/api/ai/generate',
    {
      method: 'POST',
      body: JSON.stringify({ prompt, provider }),
    }
  );
  return response.data;
}

/**
 * Validates a YAML service definition.
 */
export async function aiValidate(yaml: string): Promise<{ valid: boolean; errors: string[] }> {
  const response = await apiRequest<{ success: boolean; data: { valid: boolean; errors: string[] } }>(
    '/api/ai/validate',
    {
      method: 'POST',
      body: JSON.stringify({ yaml }),
    }
  );
  return response.data;
}

/**
 * Gets the AI configuration status.
 */
export async function getAiConfig(): Promise<AiConfigResponse> {
  const response = await apiRequest<{ success: boolean; data: AiConfigResponse }>(
    '/api/ai/config'
  );
  return response.data;
}

// ============================================================================
// Code Generation API
// ============================================================================

export interface TypeScriptGenerateRequest {
  service_name: string;
}

export interface ReactQueryGenerateRequest {
  service_name: string;
}

export interface AxiosGenerateRequest {
  service_name: string;
}

/**
 * Generates TypeScript types from a service definition.
 */
export async function generateTypeScript(serviceName: string): Promise<string> {
  const response = await apiRequest<{ success: boolean; data: { code: string } }>(
    '/api/codegen/typescript',
    {
      method: 'POST',
      body: JSON.stringify({ service_name: serviceName }),
    }
  );
  return response.data.code;
}

/**
 * Generates React Query hooks from a service definition.
 */
export async function generateReactQuery(serviceName: string): Promise<string> {
  const response = await apiRequest<{ success: boolean; data: { code: string } }>(
    '/api/codegen/react-query',
    {
      method: 'POST',
      body: JSON.stringify({ service_name: serviceName }),
    }
  );
  return response.data.code;
}

/**
 * Generates Axios client from a service definition.
 */
export async function generateAxios(serviceName: string): Promise<string> {
  const response = await apiRequest<{ success: boolean; data: { code: string } }>(
    '/api/codegen/axios',
    {
      method: 'POST',
      body: JSON.stringify({ service_name: serviceName }),
    }
  );
  return response.data.code;
}

// ============================================================================
// Configuration API
// ============================================================================

export interface UpdateConfigRequest {
  config: Record<string, any>;
}

export interface ValidateConfigResponse {
  valid: boolean;
  errors: string[];
}

/**
 * Gets the current configuration.
 */
export async function getConfig(): Promise<Record<string, any>> {
  const response = await apiRequest<{ success: boolean; data: Record<string, any> }>(
    '/api/config'
  );
  return response.data;
}

/**
 * Updates the configuration.
 */
export async function updateConfig(config: Record<string, any>): Promise<void> {
  await apiRequest('/api/config', {
    method: 'PUT',
    body: JSON.stringify({ config }),
  });
}

/**
 * Validates a configuration.
 */
export async function validateConfig(config: Record<string, any>): Promise<ValidateConfigResponse> {
  const response = await apiRequest<{ success: boolean; data: ValidateConfigResponse }>(
    '/api/config/validate',
    {
      method: 'POST',
      body: JSON.stringify({ config }),
    }
  );
  return response.data;
}

// ============================================================================
// Marketplace and Import API
// ============================================================================

import type { ImportUrlPayload, MarketplaceItem } from '@/lib/types';

/**
 * Fetches the list of marketplace items.
 */
export async function fetchMarketplace(): Promise<MarketplaceItem[]> {
  const response = await apiRequest<{ success: boolean; data: MarketplaceItem[] }>(
    '/api/marketplace'
  );
  return response.data;
}

/**
 * Imports a service from a URL.
 */
export async function importFromUrl(url: string, format?: string): Promise<{ service_name: string; yaml: string }> {
  const payload: ImportUrlPayload = { url, format };
  const response = await apiRequest<{ success: boolean; data: { service_name: string; yaml: string } }>(
    '/api/import/url',
    {
      method: 'POST',
      body: JSON.stringify(payload),
    }
  );
  return response.data;
}

// ============================================================================
// Legacy API (for backward compatibility)
// ============================================================================

/**
 * Fetches the list of all available services from the simulator backend.
 * This function is deprecated in favor of fetchSimulatorStatus.
 * @returns {Promise<ApiService[]>} A promise that resolves to an array of services.
 * @deprecated Use fetchSimulatorStatus instead to get a complete view of the simulator.
 */
export async function fetchServices(): Promise<ApiService[]> {
  const response = await fetch(`${getApiUrl()}/status`);
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
    const response = await fetch(`${getApiUrl()}/status`);
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
    const response = await fetch(`${getApiUrl()}/start`, { method: 'POST' });
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
    const response = await fetch(`${getApiUrl()}/stop`, { method: 'POST' });
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
    const response = await fetch(`${getApiUrl()}/api/simulator/validate`, {
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
  const response = await fetch(`${getApiUrl()}/api/services/create-graphql`, {
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
    const response = await fetch(`${getApiUrl()}/api/contract-testing`, {
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
    const response = await fetch(`${getApiUrl()}/api/codegen`, {
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

// ============================================================================
// IoT Twins API
// ============================================================================

export interface TwinDetailResponse {
  name: string;
  yaml: string;
  config: any;
}

/**
 * Lists all available IoT twins.
 */
export async function listTwins(): Promise<string[]> {
  const response = await apiRequest<{ success: boolean; data: string[] }>('/api/iot/twins');
  return response.data;
}

/**
 * Gets a specific twin definition.
 */
export async function getTwin(name: string): Promise<TwinDetailResponse> {
  const response = await apiRequest<{ success: boolean; data: TwinDetailResponse }>(
    `/api/iot/twins/${encodeURIComponent(name)}`
  );
  return response.data;
}

/**
 * Saves (creates or updates) a twin.
 */
export async function saveTwin(name: string, yaml: string): Promise<void> {
  await apiRequest<{ success: boolean; data: string }>(
    `/api/iot/twins/${encodeURIComponent(name)}`,
    {
      method: 'POST',
      body: JSON.stringify({ yaml }),
    }
  );
}

/**
 * Deletes a twin.
 */
export async function deleteTwin(name: string): Promise<void> {
  await apiRequest<{ success: boolean; data: string }>(
    `/api/iot/twins/${encodeURIComponent(name)}`,
    {
      method: 'DELETE',
    }
  );
}

/**
 * Uploads a CSV file for replay.
 */
export async function uploadReplayData(file: File): Promise<string> {
  const formData = new FormData();
  formData.append('file', file);

  const token = getToken();
  const headers: Record<string, string> = {};
  if (token) {
    headers['Authorization'] = `Bearer ${token}`;
  }

  const response = await fetch(`${getApiUrl()}/api/iot/upload`, {
    method: 'POST',
    headers,
    body: formData,
  });

  if (!response.ok) {
     const errorData = await response.json().catch(() => ({
      error: response.statusText,
    }));
    throw new Error(errorData.error || `Upload failed: ${response.statusText}`);
  }

  const result = await response.json();
  if (!result.success) {
      throw new Error(result.error || 'Upload failed');
  }

  return result.data;
}
