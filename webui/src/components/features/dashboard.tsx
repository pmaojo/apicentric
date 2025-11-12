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
import { Separator } from '@/components/ui/separator';
import type { Service } from '@/lib/types';
import { CheckCircle, ExternalLink, Play, Power, Square, XCircle } from 'lucide-react';
import * as React from 'react';

type DashboardProps = {
  services: Service[];
  onToggleService: (serviceId: string, status: 'running' | 'stopped') => void;
};

export function Dashboard({ services, onToggleService }: DashboardProps) {
  const runningServices = services.filter(s => s.status === 'running');
  const stoppedServices = services.filter(s => s.status === 'stopped');
  const isSimulatorRunning = runningServices.length > 0;

  return (
    <div className="flex flex-col gap-8">
      <Card>
        <CardHeader>
          <CardTitle>Simulator Status</CardTitle>
          <CardDescription>Overall status of the Apicentric simulator.</CardDescription>
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
        <h2 className="text-2xl font-bold tracking-tight">Active Services</h2>
        <div className="grid grid-cols-1 gap-6 md:grid-cols-2 lg:grid-cols-3">
          {runningServices.length > 0 ? (
            runningServices.map((service) => (
              <ServiceCard key={service.id} service={service} onToggle={onToggleService} />
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
              <ServiceCard key={service.id} service={service} onToggle={onToggleService} />
            ))
          ) : (
            <p className="text-muted-foreground col-span-full">No inactive services.</p>
          )}
        </div>
      </div>
    </div>
  );
}

function ServiceCard({ service, onToggle }: { service: Service; onToggle: (serviceId: string, status: 'running' | 'stopped') => void; }) {
  return (
    <Card className="flex flex-col">
      <CardHeader>
        <div className="flex items-start justify-between">
          <div>
            <CardTitle>{service.name}</CardTitle>
            <CardDescription>Version {service.version}</CardDescription>
          </div>
          <Badge variant={service.status === 'running' ? 'default' : 'destructive'} className={`${service.status === 'running' ? 'bg-green-500/20 text-green-400 border-green-500/30' : 'bg-red-500/20 text-red-400 border-red-500/30'}`}>
            {service.status === 'running' ? <CheckCircle className="mr-1 h-3 w-3" /> : <XCircle className="mr-1 h-3 w-3" />}
            {service.status}
          </Badge>
        </div>
      </CardHeader>
      <CardContent className="flex-grow">
        <div className="text-sm text-muted-foreground">
          <p>Port: <span className="font-mono text-foreground">{service.port}</span></p>
          <p>Endpoints: <span className="font-mono text-foreground">{service.endpoints.length}</span></p>
        </div>
      </CardContent>
      <CardFooter className="grid grid-cols-2 gap-2">
        <ServiceDetailsDialog service={service} />
        {service.status === 'running' ? (
          <Button variant="destructive" onClick={() => onToggle(service.id, 'stopped')}>
            <Square className="mr-2 h-4 w-4" /> Stop
          </Button>
        ) : (
          <Button onClick={() => onToggle(service.id, 'running')}>
            <Play className="mr-2 h-4 w-4" /> Start
          </Button>
        )}
      </CardFooter>
    </Card>
  );
}

function ServiceDetailsDialog({ service }: { service: Service }) {
  return (
    <Dialog>
      <DialogTrigger asChild>
        <Button variant="outline" className="w-full">
          View Details
          <ExternalLink className="ml-2 h-4 w-4" />
        </Button>
      </DialogTrigger>
      <DialogContent className="sm:max-w-2xl">
        <DialogHeader>
          <DialogTitle>{service.name} - v{service.version}</DialogTitle>
          <DialogDescription>
            Detailed information for the {service.name} service.
          </DialogDescription>
        </DialogHeader>
        <div className="grid gap-4 py-4">
          <div className="flex items-center gap-4">
            <Badge variant={service.status === 'running' ? 'default' : 'destructive'} className={`${service.status === 'running' ? 'bg-green-500/20 text-green-400 border-green-500/30' : 'bg-red-500/20 text-red-400 border-red-500/30'}`}>
              {service.status === 'running' ? 'Running' : 'Stopped'}
            </Badge>
            <Separator orientation="vertical" className="h-6" />
            <p className="text-sm">
              <span className="text-muted-foreground">Port: </span>
              <span className="font-mono">{service.port}</span>
            </p>
          </div>
          <Separator />
          <div className="space-y-2">
            <h4 className="font-semibold">Endpoints</h4>
            <div className="max-h-60 overflow-y-auto rounded-md border p-2">
              {service.endpoints.map((endpoint, index) => (
                <div key={index} className="flex items-center gap-2 p-2 rounded hover:bg-muted/50">
                   <Badge variant="secondary" className="w-20 justify-center font-mono">{endpoint.method}</Badge>
                   <p className="font-mono text-sm">{endpoint.path}</p>
                </div>
              ))}
            </div>
          </div>
           <div className="space-y-2">
            <h4 className="font-semibold">Service Definition</h4>
            <div className="max-h-60 overflow-y-auto rounded-md border bg-muted/50 p-4">
              <pre className="text-xs font-mono">
                <code>{service.definition}</code>
              </pre>
            </div>
          </div>
        </div>
      </DialogContent>
    </Dialog>
  )
}