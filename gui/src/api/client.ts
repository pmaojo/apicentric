import axios from 'axios';

const apiClient = axios.create({
  baseURL: '/api', // All requests will be prefixed with /api
  headers: {
    'Content-Type': 'application/json',
  },
});

// Interceptor to handle API responses
apiClient.interceptors.response.use(
  (response) => {
    // If the response from our backend has a `data` property, return it directly
    if (response.data && typeof response.data.data !== 'undefined') {
      if (response.data.success) {
        return response.data.data;
      } else {
        return Promise.reject(new Error(response.data.error || 'API Error'));
      }
    }
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

export const listServices = async () => {
  return apiClient.get('/services');
};

export const loadService = async (path: string) => {
  return apiClient.post('/services/load', { path });
};

export const saveService = async (path: string, yaml: string) => {
  return apiClient.post('/services/save', { path, yaml });
};

// --- Simulator Control ---

export const startSimulator = async (serviceId: string) => {
  return apiClient.post(`/services/${serviceId}/start`);
};

export const stopSimulator = async (serviceId: string) => {
  return apiClient.post(`/services/${serviceId}/stop`);
};

// --- Logging ---

export const getServiceLogs = async (serviceId: string, limit: number = 100) => {
  return apiClient.get(`/services/${serviceId}/logs`, { params: { limit } });
};

export default apiClient;
