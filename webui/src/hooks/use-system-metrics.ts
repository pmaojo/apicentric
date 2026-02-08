'use client';

import { useWebSocketSubscription } from '@/providers/websocket-provider';
import { useState, useEffect } from 'react';

export interface SystemMetrics {
  active_services: number;
  total_requests: number;
  requests_per_minute: number;
  average_response_time: number;
  error_rate: number;
  memory_usage: number;
  cpu_usage: number;
  uptime: number;
}

export interface MetricsUpdate {
  metrics: SystemMetrics;
  timestamp: string;
}

/**
 * Hook for real-time system metrics
 */
export function useSystemMetrics() {
  const [metrics, setMetrics] = useState<SystemMetrics>({
    active_services: 0,
    total_requests: 0,
    requests_per_minute: 0,
    average_response_time: 0,
    error_rate: 0,
    memory_usage: 0,
    cpu_usage: 0,
    uptime: 0,
  });

  const [lastUpdate, setLastUpdate] = useState<Date | null>(null);

  // Subscribe to metrics updates
  useWebSocketSubscription('metrics_update', (update: MetricsUpdate) => {
    setMetrics(update.metrics);
    setLastUpdate(new Date(update.timestamp));
  });

  return {
    metrics,
    lastUpdate,
  };
}