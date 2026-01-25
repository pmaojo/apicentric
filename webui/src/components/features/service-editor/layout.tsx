'use client';

import * as React from 'react';
import { Service } from '@/lib/types';
import { Button } from '@/components/ui/button';
import { ArrowLeft, Save, Play, Square, Loader2 } from 'lucide-react';
import { useRouter } from 'next/navigation';
import { RouteList } from './route-list';
import { RouteEditor } from './route-editor';
import { useToast } from '@/hooks/use-toast';
import { updateService, startService, stopService } from '@/services/api';
import * as yaml from 'js-yaml';

interface ServiceEditorLayoutProps {
  service: Service;
}

export interface RouteConfig {
  id: string;
  method: string;
  path: string;
  response: {
    status: number;
    body: string;
    headers: Record<string, string>;
  };
}

export function ServiceEditorLayout({ service: initialService }: ServiceEditorLayoutProps) {
  const router = useRouter();
  const { toast } = useToast();
  const [service, setService] = React.useState(initialService);
  const [routes, setRoutes] = React.useState<RouteConfig[]>([]);
  const [selectedRouteId, setSelectedRouteId] = React.useState<string | null>(null);
  const [isDirty, setIsDirty] = React.useState(false);
  const [isSaving, setIsSaving] = React.useState(false);
  const [isRunning, setIsRunning] = React.useState(service.status === 'running');

  // Parse YAML to Routes on mount
  React.useEffect(() => {
    try {
      if (!service.definition) return;
      
      const doc = yaml.load(service.definition) as any;
      if (doc && doc.endpoints) {
        const parsedRoutes = doc.endpoints.map((ep: any, index: number) => {
          let status = 200;
          let body = '{}';
          let headers = {};

          if (ep.responses) {
            // Handle array format (rare but possible in some schemas)
            if (Array.isArray(ep.responses)) {
              status = ep.responses[0]?.status || 200;
              body = ep.responses[0]?.body || '{}';
              headers = ep.responses[0]?.headers || {};
            } 
            // Handle object format (standard in our examples: { "200": { ... } })
            else if (typeof ep.responses === 'object') {
              const statusKeys = Object.keys(ep.responses);
              if (statusKeys.length > 0) {
                const firstStatus = statusKeys[0];
                const responseData = ep.responses[firstStatus];
                status = parseInt(firstStatus) || 200;
                body = responseData.body || '{}';
                headers = responseData.headers || {};
              }
            }
          }

          return {
            id: `route-${index}-${Date.now()}`,
            method: ep.method || 'GET',
            path: ep.path || '/',
            response: {
              status,
              body: typeof body === 'object' ? JSON.stringify(body, null, 2) : body,
              headers,
            }
          };
        });
        setRoutes(parsedRoutes);
        if (parsedRoutes.length > 0) {
          setSelectedRouteId(parsedRoutes[0].id);
        }
      }
    } catch (e) {
      console.error('Failed to parse YAML endpoints', e);
      toast({
        title: 'Parse Error',
        description: 'Could not parse existing YAML endpoints. Editor might be limited.',
        variant: 'destructive',
      });
    }
  }, [service.definition, toast]);

  const handleRouteUpdate = (updatedRoute: RouteConfig) => {
    setRoutes(prev => prev.map(r => r.id === updatedRoute.id ? updatedRoute : r));
    setIsDirty(true);
  };

  const handleAddRoute = () => {
    const newRoute: RouteConfig = {
      id: `new-${Date.now()}`,
      method: 'GET',
      path: '/new-route',
      response: {
        status: 200,
        body: '{}',
        headers: {}
      }
    };
    setRoutes(prev => [...prev, newRoute]);
    setSelectedRouteId(newRoute.id);
    setIsDirty(true);
  };

  const handleDeleteRoute = (id: string) => {
    setRoutes(prev => prev.filter(r => r.id !== id));
    if (selectedRouteId === id) {
      setSelectedRouteId(null);
    }
    setIsDirty(true);
  };

  const handleSave = async () => {
    setIsSaving(true);
    try {
      // Reconstruct YAML
      // We need to preserve other content from the original YAML if possible, 
      // but for now let's rebuild a basic structure or try to merge.
      // A simple approach is to read the original, update endpoints, and dump.
      
      let doc: any = {};
      try {
        doc = yaml.load(service.definition) || {};
      } catch {
        // start fresh if invalid
      }

      doc.name = service.name;
      doc.version = service.version;
      doc.endpoints = routes.map(r => {
        // Construct the responses object: { [status]: { body, headers } }
        const responses: any = {};
        responses[r.response.status] = {
            body: r.response.body,
            headers: r.response.headers, // TODO: headers editor support
            content_type: 'application/json' // Default to JSON for now
        };

        return {
            method: r.method,
            path: r.path,
            responses: responses
        };
      });

      // Preserve or update server config
      if (!doc.server) doc.server = {};
      doc.server.port = service.port;

      const newYaml = yaml.dump(doc);
      
      await updateService(service.name, newYaml);
      setService(prev => ({ ...prev, definition: newYaml }));
      setIsDirty(false);
      
      toast({ title: 'Saved', description: 'Service definition updated.' });
    } catch (e) {
      console.error(e);
      toast({ title: 'Error', description: 'Failed to save changes.', variant: 'destructive' });
    } finally {
      setIsSaving(false);
    }
  };

  const handleToggleService = async () => {
    try {
      if (isRunning) {
        await stopService(service.name);
        setIsRunning(false);
        toast({ title: 'Stopped', description: `${service.name} stopped.` });
      } else {
        await startService(service.name);
        setIsRunning(true);
        toast({ title: 'Started', description: `${service.name} started.` });
      }
    } catch (e) {
        toast({ title: 'Error', description: 'Failed to toggle service state.', variant: 'destructive' });
    }
  };

  return (
    <div className="flex flex-col h-screen bg-background">
      {/* Top Bar */}
      <header className="flex items-center justify-between px-6 py-3 border-b bg-card">
        <div className="flex items-center gap-4">
          <Button variant="ghost" size="icon" onClick={() => router.push('/')}>
            <ArrowLeft className="h-5 w-5" />
          </Button>
          <div>
            <h1 className="text-lg font-semibold">{service.name}</h1>
            <p className="text-xs text-muted-foreground">v{service.version} • Port: {service.port}</p>
          </div>
        </div>
        <div className="flex items-center gap-2">
            <Button
                size="sm"
                variant={isRunning ? "destructive" : "secondary"}
                onClick={handleToggleService}
            >
                {isRunning ? <Square className="h-4 w-4 mr-2" /> : <Play className="h-4 w-4 mr-2" />}
                {isRunning ? "Stop" : "Start"}
            </Button>
          <Button size="sm" onClick={handleSave} disabled={!isDirty || isSaving}>
            {isSaving ? <Loader2 className="h-4 w-4 mr-2 animate-spin" /> : <Save className="h-4 w-4 mr-2" />}
            Save Changes
          </Button>
        </div>
      </header>

      {/* Editor Area */}
      <div className="flex flex-1 overflow-hidden">
        {/* Sidebar */}
        <div className="w-80 border-r bg-muted/30 flex flex-col">
          <RouteList 
            routes={routes} 
            selectedId={selectedRouteId} 
            onSelect={setSelectedRouteId} 
            onAdd={handleAddRoute}
            onDelete={handleDeleteRoute}
          />
        </div>

        {/* Main Content */}
        <div className="flex-1 overflow-y-auto bg-background">
          {selectedRouteId ? (
            <RouteEditor 
              route={routes.find(r => r.id === selectedRouteId)!} 
              onChange={handleRouteUpdate} 
            />
          ) : (
            <div className="flex items-center justify-center h-full text-muted-foreground">
              Select a route to edit
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
