'use client';

import { useEffect, useRef, useState, useCallback } from 'react';
import { useQueryClient } from '@tanstack/react-query';

/**
 * @fileoverview Custom hook for WebSocket connection with automatic reconnection,
 * message type handling, and React Query cache invalidation.
 */

export interface WebSocketMessage {
  type: 'pong' | 'service_status' | 'request_log' | 'recording_capture' | 'initial_state';
  data?: any;
  timestamp?: number;
}

export interface ServiceStatusUpdate {
  service_name: string;
  status: 'stopped' | 'starting' | 'running' | 'stopping' | 'failed';
  port?: number;
  error?: string;
}

export interface RequestLogEntry {
  timestamp: string;
  service: string;
  method: string;
  path: string;
  status: number;
  duration_ms?: number;
}

export interface RecordingCapture {
  method: string;
  path: string;
  status: number;
  timestamp: string;
}

export interface InitialState {
  services: any[];
  logs: RequestLogEntry[];
  recording_status: any;
}

export interface MessageHandlers {
  onServiceStatus?: (update: ServiceStatusUpdate) => void;
  onRequestLog?: (log: RequestLogEntry) => void;
  onRecordingCapture?: (capture: RecordingCapture) => void;
  onInitialState?: (state: InitialState) => void;
}

export interface UseWebSocketOptions {
  url: string;
  onMessage?: (message: WebSocketMessage) => void;
  onOpen?: () => void;
  onClose?: () => void;
  onError?: (error: Event) => void;
  reconnectAttempts?: number;
  reconnectInterval?: number;
  enabled?: boolean;
  handlers?: MessageHandlers;
  invalidateQueries?: boolean;
}

export function useWebSocket({
  url,
  onMessage,
  onOpen,
  onClose,
  onError,
  reconnectAttempts = 5,
  reconnectInterval = 1000,
  enabled = true,
  handlers,
  invalidateQueries = true,
}: UseWebSocketOptions) {
  const [isConnected, setIsConnected] = useState(false);
  const [reconnectCount, setReconnectCount] = useState(0);
  const [lastMessage, setLastMessage] = useState<WebSocketMessage | null>(null);
  const wsRef = useRef<WebSocket | null>(null);
  const reconnectTimeoutRef = useRef<NodeJS.Timeout | null>(null);
  const pingIntervalRef = useRef<NodeJS.Timeout | null>(null);
  const queryClient = useQueryClient();

  /**
   * Handles incoming WebSocket messages by type and triggers appropriate actions.
   */
  const handleMessage = useCallback((message: WebSocketMessage) => {
    setLastMessage(message);
    
    // Call the generic onMessage handler if provided
    onMessage?.(message);

    // Handle specific message types
    switch (message.type) {
      case 'service_status':
        handlers?.onServiceStatus?.(message.data as ServiceStatusUpdate);
        
        // Invalidate service-related queries
        if (invalidateQueries) {
          queryClient.invalidateQueries({ queryKey: ['services'] });
          queryClient.invalidateQueries({ 
            queryKey: ['service', message.data?.service_name] 
          });
        }
        break;

      case 'request_log':
        handlers?.onRequestLog?.(message.data as RequestLogEntry);
        
        // Invalidate logs queries
        if (invalidateQueries) {
          queryClient.invalidateQueries({ queryKey: ['logs'] });
        }
        break;

      case 'recording_capture':
        handlers?.onRecordingCapture?.(message.data as RecordingCapture);
        
        // Invalidate recording queries
        if (invalidateQueries) {
          queryClient.invalidateQueries({ queryKey: ['recording'] });
        }
        break;

      case 'initial_state':
        handlers?.onInitialState?.(message.data as InitialState);
        
        // Invalidate all relevant queries on initial state
        if (invalidateQueries) {
          queryClient.invalidateQueries({ queryKey: ['services'] });
          queryClient.invalidateQueries({ queryKey: ['logs'] });
          queryClient.invalidateQueries({ queryKey: ['recording'] });
        }
        break;

      case 'pong':
        // Heartbeat response - no action needed
        break;

      default:
        console.warn('Unknown WebSocket message type:', message.type);
    }
  }, [onMessage, handlers, invalidateQueries, queryClient]);

  const connect = useCallback(() => {
    if (!enabled || wsRef.current?.readyState === WebSocket.OPEN) {
      return;
    }

    try {
      const ws = new WebSocket(url);
      wsRef.current = ws;

      ws.onopen = () => {
        setIsConnected(true);
        setReconnectCount(0);
        onOpen?.();

        // Start ping interval
        pingIntervalRef.current = setInterval(() => {
          if (ws.readyState === WebSocket.OPEN) {
            ws.send(JSON.stringify({ type: 'ping' }));
          }
        }, 30000); // Ping every 30 seconds
      };

      ws.onmessage = (event) => {
        try {
          const message: WebSocketMessage = JSON.parse(event.data);
          handleMessage(message);
        } catch (error) {
          console.error('Failed to parse WebSocket message:', error);
        }
      };

      ws.onerror = (error) => {
        // Silently handle connection errors during reconnection attempts
        // Only notify callback if provided
        onError?.(error);
      };

      ws.onclose = () => {
        setIsConnected(false);
        onClose?.();

        // Clear ping interval
        if (pingIntervalRef.current) {
          clearInterval(pingIntervalRef.current);
          pingIntervalRef.current = null;
        }

        // Attempt reconnection with exponential backoff (more aggressive)
        if (reconnectCount < reconnectAttempts) {
          const delay = Math.min(
            reconnectInterval * Math.pow(2, reconnectCount),
            60000 // Max 60 seconds between retries
          );
          
          reconnectTimeoutRef.current = setTimeout(() => {
            setReconnectCount((prev) => prev + 1);
            connect();
          }, delay);
        }
      };
    } catch (error) {
      console.error('Failed to create WebSocket connection:', error);
    }
  }, [url, enabled, reconnectCount, reconnectAttempts, reconnectInterval, handleMessage, onOpen, onClose, onError]);

  const disconnect = useCallback(() => {
    if (reconnectTimeoutRef.current) {
      clearTimeout(reconnectTimeoutRef.current);
      reconnectTimeoutRef.current = null;
    }

    if (pingIntervalRef.current) {
      clearInterval(pingIntervalRef.current);
      pingIntervalRef.current = null;
    }

    if (wsRef.current) {
      wsRef.current.close();
      wsRef.current = null;
    }

    setIsConnected(false);
    setReconnectCount(0);
  }, []);

  const sendMessage = useCallback((message: any) => {
    if (wsRef.current?.readyState === WebSocket.OPEN) {
      wsRef.current.send(JSON.stringify(message));
    } else {
      console.warn('WebSocket is not connected');
    }
  }, []);

  useEffect(() => {
    if (enabled) {
      connect();
    }

    return () => {
      disconnect();
    };
  }, [enabled, connect, disconnect]);

  return {
    isConnected,
    reconnectCount,
    lastMessage,
    connect,
    disconnect,
    sendMessage,
  };
}

/**
 * Convenience hook for subscribing to specific WebSocket message types.
 * This hook automatically handles message filtering and provides type-safe handlers.
 */
export function useWebSocketSubscription(
  url: string,
  handlers: MessageHandlers,
  options?: Omit<UseWebSocketOptions, 'url' | 'handlers'>
) {
  return useWebSocket({
    url,
    handlers,
    ...options,
  });
}
