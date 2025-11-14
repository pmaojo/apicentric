'use client';

import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { useSystemMetrics } from '@/hooks/use-system-metrics';
import { Activity, Clock, Cpu, MemoryStick, TrendingUp, AlertTriangle } from 'lucide-react';

export function SystemMetrics() {
  const { metrics, lastUpdate } = useSystemMetrics();

  const formatUptime = (seconds: number) => {
    const hours = Math.floor(seconds / 3600);
    const minutes = Math.floor((seconds % 3600) / 60);
    return `${hours}h ${minutes}m`;
  };

  const formatBytes = (bytes: number) => {
    const mb = bytes / (1024 * 1024);
    return `${mb.toFixed(1)} MB`;
  };

  const getErrorRateColor = (rate: number) => {
    if (rate < 1) return 'bg-green-500/20 text-green-400 border-green-500/30';
    if (rate < 5) return 'bg-yellow-500/20 text-yellow-400 border-yellow-500/30';
    return 'bg-red-500/20 text-red-400 border-red-500/30';
  };

  const getCpuColor = (usage: number) => {
    if (usage < 50) return 'text-green-500';
    if (usage < 80) return 'text-yellow-500';
    return 'text-red-500';
  };

  const getMemoryColor = (usage: number) => {
    const usageMB = usage / (1024 * 1024);
    if (usageMB < 100) return 'text-green-500';
    if (usageMB < 200) return 'text-yellow-500';
    return 'text-red-500';
  };

  return (
    <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
      <Card>
        <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
          <CardTitle className="text-sm font-medium">Active Services</CardTitle>
          <Activity className="h-4 w-4 text-muted-foreground" />
        </CardHeader>
        <CardContent>
          <div className="text-2xl font-bold">{metrics.active_services}</div>
          <p className="text-xs text-muted-foreground">
            Services currently running
          </p>
        </CardContent>
      </Card>

      <Card>
        <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
          <CardTitle className="text-sm font-medium">Total Requests</CardTitle>
          <TrendingUp className="h-4 w-4 text-muted-foreground" />
        </CardHeader>
        <CardContent>
          <div className="text-2xl font-bold">{metrics.total_requests.toLocaleString()}</div>
          <p className="text-xs text-muted-foreground">
            {metrics.requests_per_minute.toFixed(1)}/min
          </p>
        </CardContent>
      </Card>

      <Card>
        <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
          <CardTitle className="text-sm font-medium">Response Time</CardTitle>
          <Clock className="h-4 w-4 text-muted-foreground" />
        </CardHeader>
        <CardContent>
          <div className="text-2xl font-bold">{metrics.average_response_time.toFixed(0)}ms</div>
          <div className="flex items-center gap-2 mt-1">
            <Badge className={getErrorRateColor(metrics.error_rate)}>
              {metrics.error_rate.toFixed(1)}% errors
            </Badge>
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
          <CardTitle className="text-sm font-medium">System Resources</CardTitle>
          <Cpu className="h-4 w-4 text-muted-foreground" />
        </CardHeader>
        <CardContent>
          <div className="space-y-2">
            <div className="flex items-center justify-between">
              <span className="text-sm text-muted-foreground">CPU</span>
              <span className={`text-sm font-medium ${getCpuColor(metrics.cpu_usage)}`}>
                {metrics.cpu_usage.toFixed(1)}%
              </span>
            </div>
            <div className="flex items-center justify-between">
              <span className="text-sm text-muted-foreground">Memory</span>
              <span className={`text-sm font-medium ${getMemoryColor(metrics.memory_usage)}`}>
                {formatBytes(metrics.memory_usage)}
              </span>
            </div>
            <div className="flex items-center justify-between">
              <span className="text-sm text-muted-foreground">Uptime</span>
              <span className="text-sm font-medium">
                {formatUptime(metrics.uptime)}
              </span>
            </div>
          </div>
        </CardContent>
      </Card>

      {lastUpdate && (
        <Card className="md:col-span-2 lg:col-span-4">
          <CardContent className="pt-6">
            <div className="flex items-center justify-center gap-2 text-sm text-muted-foreground">
              <Activity className="h-3 w-3" />
              <span>Last updated: {lastUpdate.toLocaleTimeString()}</span>
            </div>
          </CardContent>
        </Card>
      )}
    </div>
  );
}