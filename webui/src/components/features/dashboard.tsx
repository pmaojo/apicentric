'use client';

import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle } from '@/components/ui/card';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from '@/components/ui/dialog';
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from '@/components/ui/alert-dialog';
import { Separator } from '@/components/ui/separator';
import { useToast } from '@/hooks/use-toast';
import { useWebSocketSubscription, type ServiceStatusUpdate } from '@/providers/websocket-provider';
import { SystemMetrics } from './system-metrics';
import type { Service } from '@/lib/types';
import { startService, stopService, reloadServices } from '@/services/api';
import Link from 'next/link';
import { CheckCircle, ExternalLink, Play, Power, Square, XCircle, RefreshCw, Loader2, Edit } from 'lucide-react';
import * as React from 'react';
import { useCallback, memo } from 'react';
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs"
import { ApiDocs } from "./api-docs"

type DashboardProps = {
  services: Service[];
  onServiceUpdate?: (serviceName: string, updates: Partial<Service>) => void;
};

// Moved helper components before Dashboard to avoid hoisting issues with const components

function ServiceDetailsDialog({ service }: { service: Service }) {
  return (
    <Dialog>
      <DialogTrigger asChild>
        <Button variant="outline" className="w-full">
          View Details
          <ExternalLink className="ml-2 h-4 w-4" />
        </Button>
      </DialogTrigger>
      <DialogContent className="sm:max-w-4xl max-h-[90vh] overflow-hidden flex flex-col">
        <DialogHeader>
          <DialogTitle>{service.name} - v{service.version}</DialogTitle>
          <DialogDescription>
            Detailed information for the {service.name} service.
          </DialogDescription>
        </DialogHeader>

        <Tabs defaultValue="endpoints" className="w-full flex-1 flex flex-col overflow-hidden">
          <TabsList className="grid w-full grid-cols-3">
            <TabsTrigger value="endpoints">Endpoints</TabsTrigger>
            <TabsTrigger value="definition">Definition (YAML)</TabsTrigger>
            <TabsTrigger value="docs">Live Documentation</TabsTrigger>
          </TabsList>

          <div className="flex-1 overflow-y-auto mt-4 px-1">
            <TabsContent value="endpoints" className="mt-0 space-y-4">
              <div className="flex items-center gap-4 mb-4">
                <Badge variant={service.status === 'running' ? 'default' : 'destructive'} className={`${service.status === 'running' ? 'bg-green-500/20 text-green-400 border-green-500/30' : 'bg-red-500/20 text-red-400 border-red-500/30'}`}>
                  {service.status === 'running' ? 'Running' : 'Stopped'}
                </Badge>
                <Separator orientation="vertical" className="h-6" />
                <p className="text-sm">
                  <span className="text-muted-foreground">Port: </span>
                  <span className="font-mono">{service.port}</span>
                </p>
              </div>

              <div className="space-y-2">
                <h4 className="font-semibold">Endpoints</h4>
                <div className="rounded-md border p-2">
                  {service.endpoints.length > 0 ? (
                    service.endpoints.map((endpoint, index) => (
                      <div key={index} className="flex items-center gap-2 p-2 rounded hover:bg-muted/50">
                        <Badge variant="secondary" className="w-20 justify-center font-mono">{endpoint.method}</Badge>
                        <p className="font-mono text-sm">{endpoint.path}</p>
                      </div>
                    ))
                  ) : (
                    <div className="flex flex-col items-center justify-center py-8 text-center">
                      <p className="text-sm text-muted-foreground italic">
                        {service.definition.includes('graphql:') ? 'This is a GraphQL service. Endpoints are managed via the /graphql path.' : 'No standard HTTP endpoints defined for this service.'}
                      </p>
                    </div>
                  )}
                </div>
              </div>
            </TabsContent>

            <TabsContent value="definition" className="mt-0 h-full">
              <div className="rounded-md border bg-muted/50 p-4 h-full overflow-auto">
                {service.definition ? (
                  <pre className="text-xs font-mono">
                    <code>{service.definition}</code>
                  </pre>
                ) : (
                  <p className="text-xs text-muted-foreground italic">No definition available.</p>
                )}
              </div>
            </TabsContent>

            <TabsContent value="docs" className="mt-0 h-full">
               <ApiDocs serviceName={service.name} />
            </TabsContent>
          </div>
        </Tabs>
      </DialogContent>
    </Dialog>
  )
}

// Optimization: Memoize ServiceCard to prevent re-renders of all cards when one service updates.
// This is especially important for the dashboard where status updates can happen frequently.
const ServiceCard = memo(function ServiceCard({
  service,
  onStart,
  onStop,
  isLoading,
}: {
  service: Service;
  onStart: (service: Service) => void;
  onStop: (service: Service) => void;
  isLoading: boolean;
}) {
  const [showStopDialog, setShowStopDialog] = React.useState(false);

  const handleStop = () => {
    setShowStopDialog(false);
    onStop(service);
  };

  return (
    <>
      <Card className="flex flex-col" data-testid={`service-card`} data-service-name={service.name}>
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
              data-testid="service-status"
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
        <CardFooter className="grid grid-cols-2 gap-2">
          <div className="col-span-2 grid grid-cols-2 gap-2 mb-2">
            <ServiceDetailsDialog service={service} />
            <Button variant="secondary" asChild className="w-full">
              <Link href={`/editor?service=${service.name}`}>
                <Edit className="mr-2 h-4 w-4" />
                Edit
              </Link>
            </Button>
          </div>
          {service.status === 'running' ? (
            <Button
              variant="destructive"
              onClick={() => setShowStopDialog(true)}
              disabled={isLoading}
              data-testid="stop-service-button"
            >
              {isLoading ? (
                <Loader2 className="mr-2 h-4 w-4 animate-spin" />
              ) : (
                <Square className="mr-2 h-4 w-4" />
              )}
              Stop
            </Button>
          ) : (
            <Button onClick={() => onStart(service)} disabled={isLoading} data-testid="start-service-button">
              {isLoading ? (
                <Loader2 className="mr-2 h-4 w-4 animate-spin" />
              ) : (
                <Play className="mr-2 h-4 w-4" />
              )}
              Start
            </Button>
          )}
        </CardFooter>
      </Card>

      <AlertDialog open={showStopDialog} onOpenChange={setShowStopDialog}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>Stop Service</AlertDialogTitle>
            <AlertDialogDescription>
              Are you sure you want to stop {service.name}? This will terminate all active connections.
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>Cancel</AlertDialogCancel>
            <AlertDialogAction onClick={handleStop} className="bg-destructive text-destructive-foreground hover:bg-destructive/90">
              Stop Service
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </>
  );
});

export function Dashboard({ services, onServiceUpdate }: DashboardProps) {
  const { toast } = useToast();
  const [loadingServices, setLoadingServices] = React.useState<Set<string>>(new Set());
  const [reloadingAll, setReloadingAll] = React.useState(false);

  // Subscribe to service status updates via WebSocket
  // Note: useWebSocketSubscription internally uses a ref for the callback,
  // so we don't need to pass a dependency array to avoid re-subscriptions.
  useWebSocketSubscription('service_status', (update: ServiceStatusUpdate) => {
    onServiceUpdate?.(update.service_name, {
      status: update.status as 'running' | 'stopped',
      port: update.port,
    });
    
    // Remove from loading state when status changes
    setLoadingServices((prev: Set<string>) => {
      const next = new Set(prev);
      next.delete(update.service_name);
      return next;
    });

    // Show toast notification for status changes
    toast({
      title: `Service ${update.status === 'running' ? 'Started' : 'Stopped'}`,
      description: `${update.service_name} is now ${update.status}`,
      variant: update.status === 'running' ? 'default' : 'destructive',
    });
  });

  // Optimization: specific handlers memoized to be passed to ServiceCard
  const handleStartService = useCallback(async (service: Service) => {
    setLoadingServices((prev: Set<string>) => new Set(prev).add(service.name));
    
    try {
      await startService(service.name);
      toast({
        title: 'Service Starting',
        description: `${service.name} is starting...`,
      });
    } catch (error) {
      setLoadingServices((prev: Set<string>) => {
        const next = new Set(prev);
        next.delete(service.name);
        return next;
      });
      
      toast({
        variant: 'destructive',
        title: 'Failed to Start Service',
        description: error instanceof Error ? error.message : 'An unknown error occurred',
      });
    }
  }, [toast]);

  const handleStopService = useCallback(async (service: Service) => {
    setLoadingServices((prev: Set<string>) => new Set(prev).add(service.name));
    
    try {
      await stopService(service.name);
      toast({
        title: 'Service Stopping',
        description: `${service.name} is stopping...`,
      });
    } catch (error) {
      setLoadingServices((prev: Set<string>) => {
        const next = new Set(prev);
        next.delete(service.name);
        return next;
      });
      
      toast({
        variant: 'destructive',
        title: 'Failed to Stop Service',
        description: error instanceof Error ? error.message : 'An unknown error occurred',
      });
    }
  }, [toast]);

  const handleReloadAll = useCallback(async () => {
    setReloadingAll(true);
    
    try {
      await reloadServices();
      toast({
        title: 'Services Reloading',
        description: 'All services are being reloaded...',
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
  }, [toast]);

  const runningServices = services.filter(s => s.status === 'running');
  const stoppedServices = services.filter(s => s.status === 'stopped');
  const isSimulatorRunning = runningServices.length > 0;

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
          {runningServices.length > 0 ? (
            runningServices.map((service) => (
              <ServiceCard
                key={service.id}
                service={service}
                onStart={handleStartService}
                onStop={handleStopService}
                isLoading={loadingServices.has(service.name)}
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
            stoppedServices.map((service) => (
              <ServiceCard
                key={service.id}
                service={service}
                onStart={handleStartService}
                onStop={handleStopService}
                isLoading={loadingServices.has(service.name)}
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
