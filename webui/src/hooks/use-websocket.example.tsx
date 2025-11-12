/**
 * @fileoverview Examples of using the useWebSocket hook with message type handling
 * and React Query cache invalidation.
 */

import { useWebSocket, useWebSocketSubscription } from './use-websocket';
import { useState } from 'react';

/**
 * Example 1: Basic usage with automatic cache invalidation
 */
export function BasicWebSocketExample() {
  const { isConnected, reconnectCount } = useWebSocket({
    url: 'ws://localhost:8080/ws',
    enabled: true,
    // Automatically invalidates React Query caches when messages arrive
    invalidateQueries: true,
  });

  return (
    <div>
      <p>Status: {isConnected ? 'Connected' : 'Disconnected'}</p>
      {reconnectCount > 0 && <p>Reconnect attempts: {reconnectCount}</p>}
    </div>
  );
}

/**
 * Example 2: Using message handlers for specific message types
 */
export function MessageHandlersExample() {
  const [services, setServices] = useState<any[]>([]);
  const [logs, setLogs] = useState<any[]>([]);

  const { isConnected } = useWebSocket({
    url: 'ws://localhost:8080/ws',
    handlers: {
      onServiceStatus: (update) => {
        console.log('Service status changed:', update);
        // Update local state or trigger UI updates
        setServices((prev) =>
          prev.map((s) =>
            s.name === update.service_name
              ? { ...s, status: update.status, port: update.port }
              : s
          )
        );
      },
      onRequestLog: (log) => {
        console.log('New request log:', log);
        // Add to logs list
        setLogs((prev) => [...prev, log].slice(-100)); // Keep last 100 logs
      },
      onRecordingCapture: (capture) => {
        console.log('Recording captured:', capture);
      },
      onInitialState: (state) => {
        console.log('Initial state received:', state);
        setServices(state.services);
        setLogs(state.logs);
      },
    },
  });

  return (
    <div>
      <h2>Services ({services.length})</h2>
      <ul>
        {services.map((service) => (
          <li key={service.name}>
            {service.name} - {service.status}
          </li>
        ))}
      </ul>

      <h2>Recent Logs ({logs.length})</h2>
      <ul>
        {logs.slice(-10).map((log, i) => (
          <li key={i}>
            {log.method} {log.path} - {log.status}
          </li>
        ))}
      </ul>
    </div>
  );
}

/**
 * Example 3: Using the convenience subscription hook
 */
export function SubscriptionExample() {
  const [lastServiceUpdate, setLastServiceUpdate] = useState<string>('');

  const { isConnected, lastMessage } = useWebSocketSubscription(
    'ws://localhost:8080/ws',
    {
      onServiceStatus: (update) => {
        setLastServiceUpdate(
          `${update.service_name} is now ${update.status}`
        );
      },
      onRequestLog: (log) => {
        console.log('Request:', log.method, log.path);
      },
    },
    {
      reconnectAttempts: 10,
      reconnectInterval: 2000,
    }
  );

  return (
    <div>
      <p>Connected: {isConnected ? 'Yes' : 'No'}</p>
      <p>Last update: {lastServiceUpdate}</p>
      {lastMessage && (
        <p>Last message type: {lastMessage.type}</p>
      )}
    </div>
  );
}

/**
 * Example 4: Conditional connection with custom error handling
 */
export function ConditionalConnectionExample() {
  const [enabled, setEnabled] = useState(false);
  const [errors, setErrors] = useState<string[]>([]);

  const { isConnected, connect, disconnect } = useWebSocket({
    url: 'ws://localhost:8080/ws',
    enabled,
    onOpen: () => {
      console.log('WebSocket connected successfully');
      setErrors([]);
    },
    onError: (error) => {
      console.error('WebSocket error:', error);
      setErrors((prev) => [...prev, 'Connection error occurred']);
    },
    onClose: () => {
      console.log('WebSocket connection closed');
    },
    reconnectAttempts: 3,
  });

  return (
    <div>
      <button onClick={() => setEnabled(!enabled)}>
        {enabled ? 'Disable' : 'Enable'} WebSocket
      </button>
      
      {enabled && (
        <>
          <button onClick={connect}>Reconnect</button>
          <button onClick={disconnect}>Disconnect</button>
        </>
      )}

      <p>Status: {isConnected ? 'Connected' : 'Disconnected'}</p>

      {errors.length > 0 && (
        <div>
          <h3>Errors:</h3>
          <ul>
            {errors.map((error, i) => (
              <li key={i}>{error}</li>
            ))}
          </ul>
        </div>
      )}
    </div>
  );
}

/**
 * Example 5: Sending messages through WebSocket
 */
export function SendMessageExample() {
  const { isConnected, sendMessage } = useWebSocket({
    url: 'ws://localhost:8080/ws',
  });

  const subscribeToChannel = (channel: string) => {
    sendMessage({
      type: 'subscribe',
      channels: [channel],
    });
  };

  const unsubscribeFromChannel = (channel: string) => {
    sendMessage({
      type: 'unsubscribe',
      channels: [channel],
    });
  };

  return (
    <div>
      <p>Connected: {isConnected ? 'Yes' : 'No'}</p>
      
      {isConnected && (
        <div>
          <button onClick={() => subscribeToChannel('logs')}>
            Subscribe to Logs
          </button>
          <button onClick={() => subscribeToChannel('services')}>
            Subscribe to Services
          </button>
          <button onClick={() => unsubscribeFromChannel('logs')}>
            Unsubscribe from Logs
          </button>
        </div>
      )}
    </div>
  );
}

/**
 * Example 6: Integration with React Query
 * 
 * The hook automatically invalidates React Query caches when messages arrive.
 * This ensures that your UI stays in sync with the backend state.
 * 
 * Message Type -> Invalidated Query Keys:
 * - service_status -> ['services'], ['service', service_name]
 * - request_log -> ['logs']
 * - recording_capture -> ['recording']
 * - initial_state -> ['services'], ['logs'], ['recording']
 */
export function ReactQueryIntegrationExample() {
  const { isConnected } = useWebSocket({
    url: 'ws://localhost:8080/ws',
    // Enable automatic cache invalidation (default: true)
    invalidateQueries: true,
    handlers: {
      onServiceStatus: (update) => {
        // When a service status changes, the hook automatically invalidates:
        // - queryClient.invalidateQueries({ queryKey: ['services'] })
        // - queryClient.invalidateQueries({ queryKey: ['service', update.service_name] })
        // 
        // This triggers refetch of any useQuery hooks using these keys
        console.log('Service status updated, queries invalidated');
      },
    },
  });

  return (
    <div>
      <p>WebSocket connected: {isConnected ? 'Yes' : 'No'}</p>
      <p>React Query caches are automatically invalidated on updates</p>
    </div>
  );
}

/**
 * Example 7: Disable automatic cache invalidation
 * 
 * If you want to handle cache invalidation manually, you can disable it.
 */
export function ManualCacheInvalidationExample() {
  const { isConnected } = useWebSocket({
    url: 'ws://localhost:8080/ws',
    // Disable automatic cache invalidation
    invalidateQueries: false,
    handlers: {
      onServiceStatus: (update) => {
        // Handle cache invalidation manually if needed
        // queryClient.invalidateQueries({ queryKey: ['services'] });
      },
    },
  });

  return (
    <div>
      <p>Connected: {isConnected ? 'Yes' : 'No'}</p>
    </div>
  );
}
