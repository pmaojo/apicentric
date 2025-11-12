# useWebSocket Hook

A comprehensive React hook for WebSocket connections with automatic reconnection, message type handling, and React Query cache invalidation.

## Features

- ✅ **Automatic Reconnection**: Exponential backoff reconnection strategy (up to 5 attempts by default)
- ✅ **Message Type Handling**: Built-in handlers for `service_status`, `request_log`, `recording_capture`, and `initial_state` messages
- ✅ **React Query Integration**: Automatic cache invalidation when messages arrive
- ✅ **Heartbeat/Ping**: Automatic ping/pong mechanism to keep connections alive
- ✅ **TypeScript Support**: Full type safety with TypeScript interfaces
- ✅ **Flexible API**: Support for both generic and type-specific message handlers

## Installation

The hook is already included in the project. Import it from:

```typescript
import { useWebSocket, useWebSocketSubscription } from '@/hooks/use-websocket';
```

## Basic Usage

```typescript
import { useWebSocket } from '@/hooks/use-websocket';

function MyComponent() {
  const { isConnected, reconnectCount } = useWebSocket({
    url: 'ws://localhost:8080/ws',
    enabled: true,
  });

  return (
    <div>
      <p>Status: {isConnected ? 'Connected' : 'Disconnected'}</p>
      {reconnectCount > 0 && <p>Reconnect attempts: {reconnectCount}</p>}
    </div>
  );
}
```

## API Reference

### `useWebSocket(options)`

Main hook for WebSocket connections.

#### Options

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `url` | `string` | required | WebSocket URL to connect to |
| `enabled` | `boolean` | `true` | Whether to automatically connect |
| `reconnectAttempts` | `number` | `5` | Maximum number of reconnection attempts |
| `reconnectInterval` | `number` | `1000` | Base interval for reconnection (ms) |
| `invalidateQueries` | `boolean` | `true` | Enable automatic React Query cache invalidation |
| `handlers` | `MessageHandlers` | `undefined` | Type-specific message handlers |
| `onMessage` | `function` | `undefined` | Generic message handler |
| `onOpen` | `function` | `undefined` | Connection opened callback |
| `onClose` | `function` | `undefined` | Connection closed callback |
| `onError` | `function` | `undefined` | Error callback |

#### Returns

| Property | Type | Description |
|----------|------|-------------|
| `isConnected` | `boolean` | Current connection status |
| `reconnectCount` | `number` | Number of reconnection attempts |
| `lastMessage` | `WebSocketMessage \| null` | Last received message |
| `connect` | `function` | Manually trigger connection |
| `disconnect` | `function` | Manually disconnect |
| `sendMessage` | `function` | Send a message through WebSocket |

### `useWebSocketSubscription(url, handlers, options)`

Convenience hook for subscribing to specific message types.

```typescript
const { isConnected } = useWebSocketSubscription(
  'ws://localhost:8080/ws',
  {
    onServiceStatus: (update) => console.log('Service:', update),
    onRequestLog: (log) => console.log('Log:', log),
  }
);
```

## Message Types

The hook handles the following message types automatically:

### `service_status`

Triggered when a service status changes.

```typescript
interface ServiceStatusUpdate {
  service_name: string;
  status: 'stopped' | 'starting' | 'running' | 'stopping' | 'failed';
  port?: number;
  error?: string;
}
```

**Invalidated Queries:**
- `['services']`
- `['service', service_name]`

### `request_log`

Triggered when a new request is logged.

```typescript
interface RequestLogEntry {
  timestamp: string;
  service: string;
  method: string;
  path: string;
  status: number;
  duration_ms?: number;
}
```

**Invalidated Queries:**
- `['logs']`

### `recording_capture`

Triggered when a request is captured during recording mode.

```typescript
interface RecordingCapture {
  method: string;
  path: string;
  status: number;
  timestamp: string;
}
```

**Invalidated Queries:**
- `['recording']`

### `initial_state`

Sent when a client first connects, containing the current state.

```typescript
interface InitialState {
  services: any[];
  logs: RequestLogEntry[];
  recording_status: any;
}
```

**Invalidated Queries:**
- `['services']`
- `['logs']`
- `['recording']`

### `pong`

Heartbeat response - handled automatically, no action needed.

## React Query Integration

The hook automatically invalidates React Query caches when messages arrive, ensuring your UI stays in sync with the backend state.

### Automatic Cache Invalidation

When `invalidateQueries: true` (default):

```typescript
const { isConnected } = useWebSocket({
  url: 'ws://localhost:8080/ws',
  invalidateQueries: true, // Enabled by default
});
```

This will automatically call `queryClient.invalidateQueries()` for relevant query keys when messages arrive.

### Manual Cache Invalidation

If you prefer to handle cache invalidation manually:

```typescript
const queryClient = useQueryClient();

const { isConnected } = useWebSocket({
  url: 'ws://localhost:8080/ws',
  invalidateQueries: false, // Disable automatic invalidation
  handlers: {
    onServiceStatus: (update) => {
      // Manually invalidate specific queries
      queryClient.invalidateQueries({ queryKey: ['services'] });
    },
  },
});
```

## Advanced Usage

### Custom Message Handlers

```typescript
const { isConnected } = useWebSocket({
  url: 'ws://localhost:8080/ws',
  handlers: {
    onServiceStatus: (update) => {
      console.log(`Service ${update.service_name} is now ${update.status}`);
      // Update local state, show notifications, etc.
    },
    onRequestLog: (log) => {
      console.log(`${log.method} ${log.path} - ${log.status}`);
    },
    onRecordingCapture: (capture) => {
      console.log('Captured request:', capture);
    },
    onInitialState: (state) => {
      console.log('Initial state received:', state);
    },
  },
});
```

### Conditional Connection

```typescript
const [enabled, setEnabled] = useState(false);

const { isConnected, connect, disconnect } = useWebSocket({
  url: 'ws://localhost:8080/ws',
  enabled, // Control connection with state
});

// Manually control connection
<button onClick={() => setEnabled(!enabled)}>
  {enabled ? 'Disable' : 'Enable'} WebSocket
</button>
```

### Custom Reconnection Strategy

```typescript
const { isConnected, reconnectCount } = useWebSocket({
  url: 'ws://localhost:8080/ws',
  reconnectAttempts: 10, // Try 10 times
  reconnectInterval: 2000, // Start with 2 second delay
  // Exponential backoff: 2s, 4s, 8s, 16s, 30s (capped), ...
});
```

### Sending Messages

```typescript
const { sendMessage, isConnected } = useWebSocket({
  url: 'ws://localhost:8080/ws',
});

const subscribeToChannel = (channel: string) => {
  if (isConnected) {
    sendMessage({
      type: 'subscribe',
      channels: [channel],
    });
  }
};
```

### Error Handling

```typescript
const [errors, setErrors] = useState<string[]>([]);

const { isConnected } = useWebSocket({
  url: 'ws://localhost:8080/ws',
  onError: (error) => {
    console.error('WebSocket error:', error);
    setErrors((prev) => [...prev, 'Connection error occurred']);
  },
  onClose: () => {
    console.log('Connection closed');
  },
  onOpen: () => {
    console.log('Connection established');
    setErrors([]); // Clear errors on successful connection
  },
});
```

## Requirements Compliance

This implementation satisfies the following requirements from the spec:

- **Requirement 2.1**: ✅ Handles service status updates and broadcasts to UI
- **Requirement 2.2**: ✅ Handles request log entries in real-time
- **Requirement 2.3**: ✅ Receives initial state on connection
- **Requirement 2.5**: ✅ Implements exponential backoff reconnection (up to 5 attempts)

## Implementation Details

### Exponential Backoff

The reconnection strategy uses exponential backoff with a maximum delay of 30 seconds:

```
Attempt 1: 1s delay
Attempt 2: 2s delay
Attempt 3: 4s delay
Attempt 4: 8s delay
Attempt 5: 16s delay
Attempt 6+: 30s delay (capped)
```

### Heartbeat Mechanism

The hook automatically sends ping messages every 30 seconds to keep the connection alive:

```typescript
// Automatic ping every 30 seconds
setInterval(() => {
  if (ws.readyState === WebSocket.OPEN) {
    ws.send(JSON.stringify({ type: 'ping' }));
  }
}, 30000);
```

### Connection Cleanup

The hook properly cleans up resources on unmount:
- Clears reconnection timeouts
- Clears ping intervals
- Closes WebSocket connection
- Resets connection state

## Examples

See `use-websocket.example.tsx` for comprehensive usage examples including:
- Basic usage with automatic cache invalidation
- Using message handlers for specific message types
- Using the convenience subscription hook
- Conditional connection with custom error handling
- Sending messages through WebSocket
- React Query integration patterns
- Manual cache invalidation

## Best Practices

1. **Always check `isConnected`** before sending messages
2. **Use type-specific handlers** for better code organization
3. **Enable automatic cache invalidation** for real-time UI updates
4. **Handle errors gracefully** with `onError` callback
5. **Clean up on unmount** (handled automatically by the hook)
6. **Use exponential backoff** for reconnection (default behavior)
7. **Monitor `reconnectCount`** to detect connection issues

## Troubleshooting

### Connection not establishing

- Check that the WebSocket URL is correct
- Verify the backend WebSocket server is running
- Check browser console for error messages
- Ensure `enabled` is set to `true`

### Messages not being received

- Verify `isConnected` is `true`
- Check that message handlers are properly defined
- Ensure the backend is sending messages in the correct format
- Check browser console for parsing errors

### Cache not invalidating

- Ensure `invalidateQueries` is `true` (default)
- Verify React Query is properly set up with `QueryClientProvider`
- Check that query keys match between queries and invalidation

### Too many reconnection attempts

- Adjust `reconnectAttempts` to a higher value
- Increase `reconnectInterval` for longer delays
- Check backend logs for connection issues
- Verify network connectivity

## License

This hook is part of the Apicentric project.
