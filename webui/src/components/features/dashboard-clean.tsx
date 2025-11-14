/**
 * Migrated Dashboard Component
 * 
 * This is the first component migrated to use the clean architecture.
 * It demonstrates proper separation of concerns.
 */

'use client';

import * as React from 'react';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Separator } from '@/components/ui/separator';
import { useToast } from '@/hooks/use-toast';
import { useWebSocketSubscription, type ServiceStatusUpdate } from '@/providers/websocket-provider';
import { SystemMetrics } from './system-metrics';
import type { Service } from '@/lib/types';
import { CheckCircle, ExternalLink, Play, Power, Square, XCircle, RefreshCw, Loader2 } from 'lucide-react';

// Import clean hooks that handle business logic
import {
  useServices,
  useStartService,
  useStopService,
  useServiceUpdates,
} from '@/stores/service-store-working';

type DashboardProps = {
  onServiceUpdate?: (serviceName: string, updates: Partial<Service>) => void;
};

export function DashboardClean({ onServiceUpdate }: DashboardProps) {
  const { toast } = useToast();
  const [reloadingAll, setReloadingAll] = React.useState(false);

  // ============================================================================
  // Clean Architecture: State Management via hooks
  // ============================================================================
  
  const { data: services = [], isLoading, error, refetch } = useServices() as {
    data: Service[];
    isLoading: boolean;
    error: Error | null;
    refetch: () => Promise<any>;
  };
  const startServiceMutation = useStartService();
  const stopServiceMutation = useStopService();
  
  // Real-time updates
  useServiceUpdates();

  // Subscribe to service status updates via WebSocket
  useWebSocketSubscription('service_status', (update: ServiceStatusUpdate) => {
    onServiceUpdate?.(update.service_name, {
      status: update.status as 'running' | 'stopped',
      port: update.port,
    });

    // Show toast notification for status changes
    toast({
      title: `Service ${update.status === 'running' ? 'Started' : 'Stopped'}`,
      description: `${update.service_name} is now ${update.status}`,
      variant: update.status === 'running' ? 'default' : 'destructive',
    });
  }, [onServiceUpdate, toast]);

  // ============================================================================
  // Event Handlers (only UI logic, business logic is in hooks)
  // ============================================================================

  const handleStartService = React.useCallback((service: Service) => {
    startServiceMutation.mutate(service.name);
  }, [startServiceMutation]);

  const handleStopService = React.useCallback((service: Service) => {
    stopServiceMutation.mutate(service.name);
  }, [stopServiceMutation]);

  const handleReloadAll = React.useCallback(async () => {
    setReloadingAll(true);
    try {
      await refetch();
      toast({
        title: 'Services Reloaded',
        description: 'All services have been reloaded successfully.',
      });
    } catch (error) {
      toast({
        variant: 'destructive',
        title: 'Failed to Reload Services',
        description: error instanceof Error ? error.message : 'An unknown error occurred',
      });
    } finally {
      setReloadingAll(false);
    }
  }, [refetch, toast]);

  // ============================================================================
  // Computed Values (UI logic only)
  // ============================================================================
  
  const runningServices = React.useMemo(() => 
    services.filter((s: Service) => s.status === 'running'), [services]
  );
  
  const stoppedServices = React.useMemo(() => 
    services.filter((s: Service) => s.status === 'stopped'), [services]
  );
  
  const isSimulatorRunning = runningServices.length > 0;

  // ============================================================================
  // Error State
  // ============================================================================
  
  if (error) {
    return (
      <Card>
        <CardContent className="pt-6">
          <div className="text-center text-destructive">
            <p>Error loading services: {error.message}</p>
            <Button variant="outline" className="mt-4" onClick={() => refetch()}>
              Retry
            </Button>
          </div>
        </CardContent>
      </Card>
    );
  }

  // ============================================================================
  // Render (pure UI)
  // ============================================================================
  
  return (
    <div className="flex flex-col gap-8">
      <Card>
        <CardHeader className="flex flex-row items-center justify-between">
          <div>
            <CardTitle>Simulator Status</CardTitle>
            <CardDescription>Overall status of the Apicentric simulator.</CardDescription>
          </div>
          <Button
            variant="outline"
            onClick={handleReloadAll}
            disabled={reloadingAll}
          >
            {reloadingAll ? (
              <Loader2 className="mr-2 h-4 w-4 animate-spin" />
            ) : (
              <RefreshCw className="mr-2 h-4 w-4" />
            )}
            Reload All
          </Button>
        </CardHeader>
        <CardContent className="flex items-center gap-4">
          <Power className={`h-8 w-8 ${isSimulatorRunning ? 'text-green-500' : 'text-red-500'}`} />
          <div>
            <p className="font-semibold">
              {isSimulatorRunning ? 'Simulator is running' : 'Simulator is stopped'}
            </p>
            <p className="text-sm text-muted-foreground">
              {runningServices.length} of {services.length} services are active.
            </p>
          </div>
        </CardContent>
      </Card>

      <div className="space-y-4">
        <h2 className="text-2xl font-bold tracking-tight">System Metrics</h2>
        <SystemMetrics />
      </div>

      <Separator />

      <div className="space-y-4">
        <h2 className="text-2xl font-bold tracking-tight">Active Services</h2>
        <div className="grid grid-cols-1 gap-6 md:grid-cols-2 lg:grid-cols-3">
          {isLoading ? (
            <div className="col-span-full flex items-center justify-center h-32">
              <Loader2 className="h-6 w-6 animate-spin" />
            </div>
          ) : runningServices.length > 0 ? (
            runningServices.map((service: Service) => (
              <ServiceCard
                key={service.id}
                service={service}
                onStart={handleStartService}
                onStop={handleStopService}
                isLoading={startServiceMutation.isPending || stopServiceMutation.isPending}
              />
            ))
          ) : (
            <p className="text-muted-foreground col-span-full">No active services.</p>
          )}
        </div>
      </div>
      
      <Separator />

      <div className="space-y-4">
        <h2 className="text-2xl font-bold tracking-tight">Inactive Services</h2>
        <div className="grid grid-cols-1 gap-6 md:grid-cols-2 lg:grid-cols-3">
          {stoppedServices.length > 0 ? (
            stoppedServices.map((service: Service) => (
              <ServiceCard
                key={service.id}
                service={service}
                onStart={handleStartService}
                onStop={handleStopService}
                isLoading={startServiceMutation.isPending || stopServiceMutation.isPending}
              />
            ))
          ) : (
            <p className="text-muted-foreground col-span-full">No inactive services.</p>
          )}
        </div>
      </div>
    </div>
  );
}

// ============================================================================
// Service Card Component (pure UI component)
// ============================================================================

interface ServiceCardProps {
  service: Service;
  onStart: (service: Service) => void;
  onStop: (service: Service) => void;
  isLoading: boolean;
}

function ServiceCard({ service, onStart, onStop, isLoading }: ServiceCardProps) {
  const [showStopDialog, setShowStopDialog] = React.useState(false);

  const handleStop = () => {
    setShowStopDialog(false);
    onStop(service);
  };

  return (
    <Card className="flex flex-col">
      <CardHeader>
        <div className="flex items-start justify-between">
          <div>
            <CardTitle>{service.name}</CardTitle>
            <CardDescription>Version {service.version}</CardDescription>
          </div>
          <Badge
            variant={service.status === 'running' ? 'default' : 'destructive'}
            className={`${
              service.status === 'running'
                ? 'bg-green-500/20 text-green-400 border-green-500/30'
                : 'bg-red-500/20 text-red-400 border-red-500/30'
            }`}
          >
            {isLoading ? (
              <Loader2 className="mr-1 h-3 w-3 animate-spin" />
            ) : service.status === 'running' ? (
              <CheckCircle className="mr-1 h-3 w-3" />
            ) : (
              <XCircle className="mr-1 h-3 w-3" />
            )}
            {isLoading ? 'Loading...' : service.status}
          </Badge>
        </div>
      </CardHeader>
      <CardContent className="flex-grow">
        <div className="text-sm text-muted-foreground">
          <p>
            Port: <span className="font-mono text-foreground">{service.port}</span>
          </p>
          <p>
            Endpoints: <span className="font-mono text-foreground">{service.endpoints.length}</span>
          </p>
        </div>
      </CardContent>
      <div className="p-6 pt-0 grid grid-cols-1 gap-2">
        {service.status === 'running' ? (
          <Button
            variant="destructive"
            onClick={handleStop}
            disabled={isLoading}
          >
            {isLoading ? (
              <Loader2 className="mr-2 h-4 w-4 animate-spin" />
            ) : (
              <Square className="mr-2 h-4 w-4" />
            )}
            Stop
          </Button>
        ) : (
          <Button onClick={() => onStart(service)} disabled={isLoading}>
            {isLoading ? (
              <Loader2 className="mr-2 h-4 w-4 animate-spin" />
            ) : (
              <Play className="mr-2 h-4 w-4" />
            )}
            Start
          </Button>
        )}
      </div>
    </Card>
  );
}