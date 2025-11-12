# WebSocket Standard - Apicentric Frontend

## Problema Resuelto

**Antes:** Cada componente creaba su propia conexión WebSocket
- ❌ Múltiples conexiones simultáneas
- ❌ Agotamiento de recursos (ERR_INSUFFICIENT_RESOURCES)
- ❌ Logs de error constantes
- ❌ Difícil de mantener

**Ahora:** Una única conexión compartida a nivel de aplicación
- ✅ Una sola conexión WebSocket
- ✅ Uso eficiente de recursos
- ✅ Sin logs de error
- ✅ Fácil de mantener y extender

## Arquitectura

```
App Layout (Root)
  └── WebSocketProvider (Single WS connection)
       ├── Dashboard (subscribes to service_status)
       ├── LogsViewer (subscribes to request_log)
       └── Other components (subscribe as needed)
```

## Uso

### 1. Wrap your app with WebSocketProvider

```tsx
// app/layout.tsx or app/page.tsx
import { WebSocketProvider } from '@/contexts/websocket-context';

export default function Layout({ children }) {
  return (
    <WebSocketProvider>
      {children}
    </WebSocketProvider>
  );
}
```

### 2. Subscribe to WebSocket events in components

```tsx
// components/dashboard.tsx
import { useWebSocketContext } from '@/contexts/websocket-context';

export function Dashboard() {
  const { isConnected, onServiceStatus } = useWebSocketContext();
  
  useEffect(() => {
    // Subscribe to service status updates
    const unsubscribe = onServiceStatus((update) => {
      console.log('Service status changed:', update);
      // Update your state here
    });
    
    // Cleanup on unmount
    return unsubscribe;
  }, [onServiceStatus]);
  
  return <div>Connected: {isConnected ? 'Yes' : 'No'}</div>;
}
```

## Benefits

1. **Single Connection:** Only one WebSocket connection for entire app
2. **Optional:** WebSocket is optional - app works without it
3. **Type-Safe:** Full TypeScript support
4. **Clean Unsubscribe:** Automatic cleanup on component unmount
5. **No Errors:** Silent error handling, no console spam

## Current Status

WebSocket is currently **disabled** (`enabled: false`) to prevent resource exhaustion.
Once backend connection pooling is improved, set `enabled: true` in the provider.
