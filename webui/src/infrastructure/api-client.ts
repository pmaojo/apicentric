/**
 * API Client - Infrastructure Layer
 * 
 * Low-level HTTP client that handles authentication, error handling, and request/response transformation.
 * This is the only place that knows about the specific API contract.
 */

// Base API configuration
const API_BASE_URL = process.env.NEXT_PUBLIC_API_URL || 'http://localhost:8080';

// Request/Response types
interface ApiResponse<T = any> {
  success: boolean;
  data?: T;
  message?: string;
  errors?: string[];
}

interface ApiError {
  message: string;
  code?: string;
  status?: number;
}

class ApiClientError extends Error {
  constructor(
    message: string,
    public status?: number,
    public code?: string
  ) {
    super(message);
    this.name = 'ApiClientError';
  }
}

/**
 * Low-level API client for HTTP requests
 */
export class ApiClient {
  private baseUrl: string;
  private defaultHeaders: Record<string, string>;

  constructor(baseUrl: string = API_BASE_URL) {
    this.baseUrl = baseUrl.replace(/\/$/, ''); // Remove trailing slash
    this.defaultHeaders = {
      'Content-Type': 'application/json',
    };
  }

  /**
   * Make an authenticated request
   */
  async request<T = any>(
    endpoint: string,
    options: RequestInit = {}
  ): Promise<T> {
    const url = `${this.baseUrl}${endpoint.startsWith('/') ? endpoint : `/${endpoint}`}`;
    
    // Merge headers
    const headers = {
      ...this.defaultHeaders,
      ...options.headers,
    };

    // Add authentication if available
    const token = this.getAuthToken();
    if (token) {
      headers['Authorization'] = `Bearer ${token}`;
    }

    const config: RequestInit = {
      ...options,
      headers,
    };

    try {
      const response = await fetch(url, config);
      
      // Handle non-JSON responses (like file downloads)
      const contentType = response.headers.get('content-type');
      if (!contentType?.includes('application/json')) {
        if (!response.ok) {
          throw new ApiClientError(
            `HTTP ${response.status}: ${response.statusText}`,
            response.status
          );
        }
        return response.text() as T;
      }

      // Parse JSON response
      const data: ApiResponse<T> = await response.json();

      // Handle API-level errors
      if (!response.ok) {
        const errorMessage = data.message || `HTTP ${response.status}: ${response.statusText}`;
        const errors = data.errors?.join(', ') || '';
        throw new ApiClientError(
          errors ? `${errorMessage}. ${errors}` : errorMessage,
          response.status,
          data.message
        );
      }

      // Handle application-level errors
      if (!data.success && data.message) {
        throw new ApiClientError(data.message, response.status);
      }

      return data.data || data as T;
    } catch (error) {
      if (error instanceof ApiClientError) {
        throw error;
      }

      // Network or other errors
      throw new ApiClientError(
        error instanceof Error ? error.message : 'Network request failed'
      );
    }
  }

  /**
   * GET request
   */
  async get<T = any>(endpoint: string, params?: Record<string, any>): Promise<T> {
    let url = endpoint;
    if (params) {
      const searchParams = new URLSearchParams();
      Object.entries(params).forEach(([key, value]) => {
        if (value !== undefined && value !== null) {
          searchParams.append(key, String(value));
        }
      });
      const queryString = searchParams.toString();
      if (queryString) {
        url += `?${queryString}`;
      }
    }

    return this.request<T>(url, { method: 'GET' });
  }

  /**
   * POST request
   */
  async post<T = any>(endpoint: string, data?: any): Promise<T> {
    return this.request<T>(endpoint, {
      method: 'POST',
      body: data ? JSON.stringify(data) : undefined,
    });
  }

  /**
   * PUT request
   */
  async put<T = any>(endpoint: string, data?: any): Promise<T> {
    return this.request<T>(endpoint, {
      method: 'PUT',
      body: data ? JSON.stringify(data) : undefined,
    });
  }

  /**
   * DELETE request
   */
  async delete<T = any>(endpoint: string): Promise<T> {
    return this.request<T>(endpoint, { method: 'DELETE' });
  }

  /**
   * Get authentication token from storage
   */
  private getAuthToken(): string | null {
    if (typeof window === 'undefined') return null;
    
    try {
      return localStorage.getItem('apicentric_token');
    } catch {
      return null;
    }
  }

  /**
   * Set authentication token
   */
  setAuthToken(token: string): void {
    if (typeof window === 'undefined') return;
    
    try {
      localStorage.setItem('apicentric_token', token);
    } catch (error) {
      console.warn('Failed to store auth token:', error);
    }
  }

  /**
   * Clear authentication token
   */
  clearAuthToken(): void {
    if (typeof window === 'undefined') return;
    
    try {
      localStorage.removeItem('apicentric_token');
    } catch (error) {
      console.warn('Failed to clear auth token:', error);
    }
  }

  /**
   * Check if user is authenticated
   */
  isAuthenticated(): boolean {
    return !!this.getAuthToken();
  }
}

// Export singleton instance
export const apiClient = new ApiClient();

// Legacy export for compatibility
export const apiRequest = <T = any>(endpoint: string, options?: RequestInit): Promise<T> => {
  return apiClient.request<T>(endpoint, options);
};