'use client';

import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from '@/components/ui/tooltip';
import { useWebSocket } from '@/providers/websocket-provider';
import { Wifi, WifiOff, RefreshCw, AlertCircle } from 'lucide-react';

export function WebSocketStatus() {
  const { isConnected, connectionState, lastError, reconnect } = useWebSocket();

  const getStatusInfo = () => {
    switch (connectionState) {
      case 'connected':
        return {
          icon: <Wifi className="h-3 w-3" />,
          label: 'Connected',
          variant: 'default' as const,
          className: 'bg-green-500/20 text-green-400 border-green-500/30',
        };
      case 'connecting':
        return {
          icon: <RefreshCw className="h-3 w-3 animate-spin" />,
          label: 'Connecting',
          variant: 'secondary' as const,
          className: 'bg-yellow-500/20 text-yellow-400 border-yellow-500/30',
        };
      case 'error':
        return {
          icon: <AlertCircle className="h-3 w-3" />,
          label: 'Error',
          variant: 'destructive' as const,
          className: 'bg-red-500/20 text-red-400 border-red-500/30',
        };
      case 'disconnected':
      default:
        return {
          icon: <WifiOff className="h-3 w-3" />,
          label: 'Disconnected',
          variant: 'secondary' as const,
          className: 'bg-gray-500/20 text-gray-400 border-gray-500/30',
        };
    }
  };

  const statusInfo = getStatusInfo();

  const tooltipContent = (
    <div className="space-y-1">
      <p className="font-medium">WebSocket Status: {statusInfo.label}</p>
      {lastError && (
        <p className="text-xs text-red-400">Error: {lastError}</p>
      )}
      {connectionState === 'connected' && (
        <p className="text-xs text-muted-foreground">
          Real-time updates active
        </p>
      )}
      {(connectionState === 'error' || connectionState === 'disconnected') && (
        <p className="text-xs text-muted-foreground">
          Click to reconnect
        </p>
      )}
    </div>
  );

  return (
    <TooltipProvider>
      <Tooltip>
        <TooltipTrigger asChild>
          {connectionState === 'error' || connectionState === 'disconnected' ? (
            <Button
              variant="ghost"
              size="sm"
              onClick={reconnect}
              className="h-8 px-2 gap-1.5"
            >
              {statusInfo.icon}
              <span className="text-xs">{statusInfo.label}</span>
            </Button>
          ) : (
            <div className="flex items-center gap-1.5 px-2 py-1 rounded-md">
              {statusInfo.icon}
              <span className="text-xs">{statusInfo.label}</span>
            </div>
          )}
        </TooltipTrigger>
        <TooltipContent>
          {tooltipContent}
        </TooltipContent>
      </Tooltip>
    </TooltipProvider>
  );
}