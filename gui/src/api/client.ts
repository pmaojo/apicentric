import axios, { AxiosResponse } from 'axios';

// --- Type Definitions ---

export interface Service {
  id: string;
  name: string;
  description?: string;
  version: string;
  schema_path: string;
  is_running: boolean;
  port: number | null;
}

export interface ConnectServiceRequest {
  peer: string;
  service: string;
  port: number;
  token?: string;
}

// --- API Client Setup ---

const apiClient = axios.create({
  baseURL: '/api', // All requests will be prefixed with /api
  headers: {
    'Content-Type': 'application/json',
  },
});

// Interceptor to handle API responses and extract data
apiClient.interceptors.response.use(
  (response: AxiosResponse) => {
    // The backend sends data directly, so we return response.data
    return response.data;
  },
  (error) => {
    // Handle network errors or other issues
    const errorMessage =
      error.response?.data?.error ||
      error.message ||
      'An unexpected error occurred';
    return Promise.reject(new Error(errorMessage));
  }
);

// --- Service Management ---

export const listServices = async (): Promise<Service[]> => {
  return apiClient.get('/services');
};

export const loadService = async (path: string): Promise<string> => {
  return apiClient.post('/services/load', { path });
};

export const saveService = async (path: string, yaml: string): Promise<void> => {
  return apiClient.post('/services/save', { path, yaml });
};

// --- Simulator Control ---

// TODO: Implement in backend
export const startSimulator = async (serviceId: string): Promise<void> => {
  // return apiClient.post(`/services/${serviceId}/start`);
  console.log(`startSimulator(${serviceId})`);
  return Promise.resolve();
};

// TODO: Implement in backend
export const stopSimulator = async (serviceId: string): Promise<void> => {
  // return apiClient.post(`/services/${serviceId}/stop`);
  console.log(`stopSimulator(${serviceId})`);
  return Promise.resolve();
};

// TODO: Implement in backend
export const shareService = async (serviceName: string): Promise<[string, string]> => {
  console.log(`shareService(${serviceName})`);
  return Promise.resolve(['mock_peer_id', 'mock_token']);
};

// TODO: Implement in backend
export const connectService = async (req: ConnectServiceRequest): Promise<void> => {
  console.log('connectService', req);
  return Promise.resolve();
};

// --- Type Generation ---

// TODO: Implement in backend
export const exportTypes = async (serviceId: string, language: 'typescript' | 'react-query'): Promise<string> => {
  console.log(`exportTypes(${serviceId}, ${language})`);
  return Promise.resolve(`// Mock types for ${serviceId}`);
};


// --- Logging ---

// TODO: Implement in backend
export const getServiceLogs = async (serviceId: string, limit: number = 100): Promise<string[]> => {
  // return apiClient.get(`/services/${serviceId}/logs`, { params: { limit } });
  console.log(`getServiceLogs(${serviceId}, ${limit})`);
  return Promise.resolve([`Mock log for ${serviceId}`]);
};

export default apiClient;
