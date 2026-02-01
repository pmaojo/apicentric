'use client';

import React, { createContext, useContext, useEffect, useRef, useState, useCallback } from 'react';
import { logger } from '../infrastructure/logger';

/**
 * WebSocket message types from the Rust backend
 */
export type WebSocketMessage = 
  | { type: 'service_status'; data: ServiceStatusUpdate }
  | { type: 'request_log'; data: RequestLogEntry }
  | { type: 'recording_capture'; data: RecordingCapture }
  | { type: 'ping'; data?: any }
  | { type: 'pong'; data?: any };

export interface ServiceStatusUpdate {
  service_name: string;
  status: 'running' | 'stopped';
  port?: number;
  timestamp: string;
}

// Re-export type from api.ts to avoid conflicts
import type { RequestLogEntry as ApiRequestLogEntry } from '@/services/api';

// Extend the API type if needed, or just use it directly
// The WebSocket version has some different fields naming
export interface WebSocketRequestLogEntry {
  id: string;
  timestamp: string;
  method: string;
  path: string;
  status: number;
  duration_ms: number; // Renamed from response_time to match ApiRequestLogEntry
  service: string; // Renamed from service_name
  request_headers?: Record<string, string>; // Renamed from headers
  request_body?: string; // Renamed from body
  response_headers?: Record<string, string>;
  response_body?: string;
}

export type RequestLogEntry = WebSocketRequestLogEntry;

export interface RecordingCapture {
  method: string;
  path: string;
  headers: Record<string, string>;
  body?: string;
  response_status: number;
  response_headers: Record<string, string>;
  response_body?: string;
}

export interface WebSocketContextValue {
  isConnected: boolean;
  connectionState: 'connecting' | 'connected' | 'disconnected' | 'error';
  lastError: string | null;
  subscribe: (messageType: string, callback: (data: any) => void) => () => void;
  send: (message: any) => void;
  reconnect: () => void;
}

const WebSocketContext = createContext<WebSocketContextValue | null>(null);

interface WebSocketProviderProps {
  children: React.ReactNode;
  url?: string;
  enabled?: boolean;
}

/**
 * Single WebSocket connection provider that manages the connection
 * and allows components to subscribe to specific message types
 */
export function WebSocketProvider({ 
  children, 
  url = 'ws://localhost:8080/ws',
  enabled = true 
}: WebSocketProviderProps) {
  const [isConnected, setIsConnected] = useState(false);
  const [connectionState, setConnectionState] = useState<'connecting' | 'connected' | 'disconnected' | 'error'>('disconnected');
  const [lastError, setLastError] = useState<string | null>(null);
  
  const wsRef = useRef<WebSocket | null>(null);
  const subscribersRef = useRef<Map<string, Set<(data: any) => void>>>(new Map());
  const reconnectTimeoutRef = useRef<NodeJS.Timeout | null>(null);
  const reconnectAttemptsRef = useRef(0);
  const maxReconnectAttempts = 5;
  const baseReconnectDelay = 1000; // 1 second

  /**
   * Subscribe to a specific message type
   */
  const subscribe = useCallback((messageType: string, callback: (data: any) => void) => {
    if (!subscribersRef.current.has(messageType)) {
      subscribersRef.current.set(messageType, new Set());
    }
    subscribersRef.current.get(messageType)!.add(callback);

    // Return unsubscribe function
    return () => {
      const subscribers = subscribersRef.current.get(messageType);
      if (subscribers) {
        subscribers.delete(callback);
        if (subscribers.size === 0) {
          subscribersRef.current.delete(messageType);
        }
      }
    };
  }, []);

  /**
   * Send a message through the WebSocket
   */
  const send = useCallback((message: any) => {
    if (wsRef.current && wsRef.current.readyState === WebSocket.OPEN) {
      wsRef.current.send(JSON.stringify(message));
    } else {
      logger.warn('WebSocket is not connected. Cannot send message:', message);
    }
  }, []);

  /**
   * Handle incoming WebSocket messages
   */
  const handleMessage = useCallback((event: MessageEvent) => {
    try {
      const message: WebSocketMessage = JSON.parse(event.data);
      
      // Handle ping/pong for connection health
      if (message.type === 'ping') {
        send({ type: 'pong', data: message.data });
        return;
      }

      // Notify subscribers
      const subscribers = subscribersRef.current.get(message.type);
      if (subscribers) {
        subscribers.forEach(callback => {
          try {
            callback(message.data);
          } catch (error) {
            logger.error(`Error in WebSocket subscriber for type ${message.type}:`, error);
          }
        });
      }
    } catch (error) {
      logger.error('Failed to parse WebSocket message:', error);
    }
  }, [send]);

  /**
   * Calculate reconnect delay with exponential backoff
   */
  const getReconnectDelay = useCallback(() => {
    return Math.min(baseReconnectDelay * Math.pow(2, reconnectAttemptsRef.current), 30000);
  }, []);

  /**
   * Connect to WebSocket
   */
  const connect = useCallback(() => {
    if (!enabled) return;

    if (wsRef.current && wsRef.current.readyState === WebSocket.CONNECTING) {
      return; // Already connecting
    }

    try {
      setConnectionState('connecting');
      setLastError(null);

      const ws = new WebSocket(url);
      wsRef.current = ws;

      ws.onopen = () => {
        logger.log('✅ WebSocket connected');
        setIsConnected(true);
        setConnectionState('connected');
        reconnectAttemptsRef.current = 0;
        setLastError(null);
      };

      ws.onmessage = handleMessage;

      ws.onclose = (event) => {
        logger.log('WebSocket closed:', event.code, event.reason);
        setIsConnected(false);
        setConnectionState('disconnected');
        wsRef.current = null;

        // Only attempt to reconnect if it wasn't a manual close and we haven't exceeded max attempts
        if (enabled && event.code !== 1000 && reconnectAttemptsRef.current < maxReconnectAttempts) {
          const delay = getReconnectDelay();
          logger.log(`Reconnecting in ${delay}ms (attempt ${reconnectAttemptsRef.current + 1}/${maxReconnectAttempts})`);
          
          reconnectTimeoutRef.current = setTimeout(() => {
            reconnectAttemptsRef.current++;
            connect();
          }, delay);
        } else if (reconnectAttemptsRef.current >= maxReconnectAttempts) {
          setConnectionState('error');
          setLastError('Maximum reconnection attempts reached');
        }
      };

      ws.onerror = (error) => {
        logger.error('WebSocket error:', error);
        setConnectionState('error');
        setLastError('WebSocket connection error');
      };

    } catch (error) {
      logger.error('Failed to create WebSocket connection:', error);
      setConnectionState('error');
      setLastError(error instanceof Error ? error.message : 'Unknown connection error');
    }
  }, [enabled, url, handleMessage, getReconnectDelay]);

  /**
   * Disconnect from WebSocket
   */
  const disconnect = useCallback(() => {
    if (reconnectTimeoutRef.current) {
      clearTimeout(reconnectTimeoutRef.current);
      reconnectTimeoutRef.current = null;
    }

    if (wsRef.current) {
      wsRef.current.close(1000, 'Manual disconnect');
      wsRef.current = null;
    }

    setIsConnected(false);
    setConnectionState('disconnected');
    reconnectAttemptsRef.current = 0;
  }, []);

  /**
   * Manual reconnect function
   */
  const reconnect = useCallback(() => {
    disconnect();
    reconnectAttemptsRef.current = 0;
    setTimeout(connect, 100);
  }, [disconnect, connect]);

  // Connect on mount and when enabled changes
  useEffect(() => {
    if (enabled) {
      connect();
    } else {
      disconnect();
    }

    return disconnect;
  }, [enabled, connect, disconnect]);

  // Cleanup on unmount
  useEffect(() => {
    const subscribers = subscribersRef.current;
    return () => {
      disconnect();
      subscribers.clear();
    };
  }, [disconnect]);

  const contextValue: WebSocketContextValue = {
    isConnected,
    connectionState,
    lastError,
    subscribe,
    send,
    reconnect,
  };

  return (
    <WebSocketContext.Provider value={contextValue}>
      {children}
    </WebSocketContext.Provider>
  );
}

/**
 * Hook to use the WebSocket context
 */
export function useWebSocket() {
  const context = useContext(WebSocketContext);
  if (!context) {
    throw new Error('useWebSocket must be used within a WebSocketProvider');
  }
  return context;
}

/**
 * Hook to subscribe to specific WebSocket message types
 */
export function useWebSocketSubscription<T = any>(
  messageType: string,
  callback: (data: T) => void
) {
  const { subscribe } = useWebSocket();
  const callbackRef = useRef(callback);

  useEffect(() => {
    callbackRef.current = callback;
  });

  useEffect(() => {
    const handler = (data: T) => {
      if (callbackRef.current) {
        callbackRef.current(data);
      }
    };
    const unsubscribe = subscribe(messageType, handler);
    return unsubscribe;
  }, [subscribe, messageType]);
}