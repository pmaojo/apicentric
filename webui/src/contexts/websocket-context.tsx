'use client';

import * as React from 'react';
import { useWebSocket, type WebSocketMessage, type ServiceStatusUpdate } from '@/hooks/use-websocket';

interface WebSocketContextValue {
  isConnected: boolean;
  lastMessage: WebSocketMessage | null;
  sendMessage: (message: any) => void;
  // Typed message handlers
  onServiceStatus: (callback: (update: ServiceStatusUpdate) => void) => () => void;
  onRequestLog: (callback: (log: any) => void) => () => void;
}

const WebSocketContext = React.createContext<WebSocketContextValue | null>(null);

export function WebSocketProvider({ children }: { children: React.ReactNode }) {
  const [lastMessage, setLastMessage] = React.useState<WebSocketMessage | null>(null);
  const serviceStatusCallbacks = React.useRef<Set<(update: ServiceStatusUpdate) => void>>(new Set());
  const requestLogCallbacks = React.useRef<Set<(log: any) => void>>(new Set());
  
  const WS_URL = typeof window !== 'undefined' 
    ? process.env.NEXT_PUBLIC_WS_URL || 'ws://localhost:8080/ws'
    : 'ws://localhost:8080/ws';

  const { isConnected, sendMessage } = useWebSocket({
    url: WS_URL,
    enabled: false, // Disabled - WebSocket is optional for now
    onMessage: (message) => {
      setLastMessage(message);
      
      // Dispatch to registered callbacks
      if (message.type === 'service_status' && message.data) {
        serviceStatusCallbacks.current.forEach(cb => cb(message.data as ServiceStatusUpdate));
      } else if (message.type === 'request_log' && message.data) {
        requestLogCallbacks.current.forEach(cb => cb(message.data));
      }
    },
    onError: () => {
      // Silently handle errors - WebSocket is optional
    },
  });

  // Register callback for service status updates
  const onServiceStatus = React.useCallback((callback: (update: ServiceStatusUpdate) => void) => {
    serviceStatusCallbacks.current.add(callback);
    return () => {
      serviceStatusCallbacks.current.delete(callback);
    };
  }, []);

  // Register callback for request logs
  const onRequestLog = React.useCallback((callback: (log: any) => void) => {
    requestLogCallbacks.current.add(callback);
    return () => {
      requestLogCallbacks.current.delete(callback);
    };
  }, []);

  return (
    <WebSocketContext.Provider 
      value={{ 
        isConnected, 
        lastMessage, 
        sendMessage,
        onServiceStatus,
        onRequestLog,
      }}
    >
      {children}
    </WebSocketContext.Provider>
  );
}

export function useWebSocketContext() {
  const context = React.useContext(WebSocketContext);
  if (!context) {
    // Return a no-op implementation if not in provider
    // This makes WebSocket truly optional
    return {
      isConnected: false,
      lastMessage: null,
      sendMessage: () => {},
      onServiceStatus: () => () => {},
      onRequestLog: () => () => {},
    };
  }
  return context;
}
