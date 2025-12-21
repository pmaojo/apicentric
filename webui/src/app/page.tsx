'use client';

import { useState, useEffect, lazy, Suspense, useMemo, useCallback, memo } from 'react';
import type { ApiService, View, Service, SimulatorStatus } from '@/lib/types';
import { MainLayout } from '@/components/layout/main-layout';
import { Dashboard } from '@/components/features/dashboard';
import { QueryClient, QueryClientProvider, useQuery, useMutation } from '@tanstack/react-query';
import { fetchSimulatorStatus, startSimulator, stopSimulator } from '@/services/api';
import { Loader2 } from 'lucide-react';
import { useToast } from '@/hooks/use-toast';
import { BackendConnectionError } from '@/components/features/backend-connection-error';

// Lazy load heavy components for code splitting
const ServiceManagement = lazy(() => import('@/components/features/service-management').then(m => ({ default: m.ServiceManagement })));
const LogsViewer = lazy(() => import('@/components/features/logs-viewer').then(m => ({ default: m.LogsViewer })));
const AiGenerator = lazy(() => import('@/components/features/ai-generator').then(m => ({ default: m.AiGenerator })));
const PluginGenerator = lazy(() => import('@/components/features/plugin-generator').then(m => ({ default: m.PluginGenerator })));
const CodeGenerator = lazy(() => import('@/components/features/code-generator').then(m => ({ default: m.CodeGenerator })));
const ContractTesting = lazy(() => import('@/components/features/contract-testing').then(m => ({ default: m.ContractTesting })));
const Recording = lazy(() => import('@/components/features/recording').then(m => ({ default: m.Recording })));
const Configuration = lazy(() => import('@/components/features/configuration').then(m => ({ default: m.Configuration })));

// Loading fallback component
const ComponentLoader = () => (
  <div className="flex items-center justify-center h-full">
    <Loader2 className="h-8 w-8 animate-spin text-primary" />
  </div>
);

/**
 * @fileoverview The main entry point component for the Apicentric UI.
 * It manages the overall layout, state, and view switching for the application.
 */

const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      retry: 1, // Minimize retries to show error state faster
      retryDelay: 1000,
    }
  }
});

let newServiceIdCounter = 0;

/**
 * Renders the main content of the application, including the layout and active view.
 * This component fetches the initial service data and handles all top-level state management,
 * such as the active view and the list of services.
 * @returns {React.ReactElement} The rendered application content.
 */
function AppContent() {
  const [activeView, setActiveView] = useState<View>('dashboard');
  const { data: simulatorStatus, isLoading, error, refetch } = useQuery<SimulatorStatus, Error>({
    queryKey: ['simulatorStatus'],
    queryFn: fetchSimulatorStatus,
    refetchInterval: (data) => data ? 10000 : false, // Only poll if we have a successful connection
  });
  const [services, setServices] = useState<Service[]>([]);
  const { toast } = useToast();

  // Memoize the mapped services to avoid unnecessary recalculations
  const mappedServices = useMemo(() => {
    if (!simulatorStatus) return [];
    return simulatorStatus.active_services.map((apiService, index): Service => ({
      id: apiService.id || `service-${index}-${Date.now()}`,
      name: apiService.name,
      status: (apiService.is_running ? 'running' : 'stopped') as 'running' | 'stopped',
      port: apiService.port,
      version: apiService.version || '1.0.0',
      definition: apiService.definition || '',
      endpoints: apiService.endpoints?.map(ep => ({
        ...ep,
        method: ep.method as 'GET' | 'POST' | 'PUT' | 'DELETE' | 'PATCH'
      })) || [],
    }));
  }, [simulatorStatus]);

  useEffect(() => {
    if (mappedServices.length > 0) {
      setServices(mappedServices);
    }
  }, [mappedServices]);

  const startMutation = useMutation({
    mutationFn: startSimulator,
    onSuccess: () => {
      toast({
        title: 'Simulator Started',
        description: 'The API simulator has been started successfully.',
      });
      refetch();
    },
    onError: (err) => {
      toast({
        variant: 'destructive',
        title: 'Failed to Start Simulator',
        description: err.message,
      });
    },
  });

  const stopMutation = useMutation({
    mutationFn: stopSimulator,
    onSuccess: () => {
      toast({
        title: 'Simulator Stopped',
        description: 'The API simulator has been stopped.',
      });
      refetch();
    },
    onError: (err) => {
      toast({
        variant: 'destructive',
        title: 'Failed to Stop Simulator',
        description: err.message,
      });
    },
  });

  // Memoize callbacks to prevent unnecessary re-renders
  const handleToggleAllServices = useCallback(() => {
    if (simulatorStatus?.is_active) {
      stopMutation.mutate();
    } else {
      startMutation.mutate();
    }
  }, [simulatorStatus?.is_active, stopMutation, startMutation]);
  
  const handleToggleService = useCallback((serviceId: string, status: 'running' | 'stopped') => {
    // This would ideally be a per-service start/stop endpoint
    // For now, we just toggle the whole simulator
    handleToggleAllServices();
  }, [handleToggleAllServices]);

  /**
   * Adds a new service to the list after parsing its YAML definition.
   * @param {object} serviceData - The data for the new service from the creation form.
   * @param {string} serviceData.name - The name of the service.
   * @param {string} serviceData.version - The version of the service.
   * @param {number} serviceData.port - The port for the service.
   * @param {string} serviceData.definition - The raw YAML definition string.
   */
  const handleAddService = useCallback(async (serviceData: { name: string, version: string, port: number, definition: string }) => {
    try {
      const yamlModule = await import('js-yaml');
      const doc: any = yamlModule.default.load(serviceData.definition);
      
      const newService: Service = {
        id: `new-service-${Date.now()}-${newServiceIdCounter++}`,
        name: doc.name || serviceData.name,
        version: doc.version || serviceData.version,
        status: 'stopped',
        port: doc.server?.port || serviceData.port,
        definition: serviceData.definition,
        endpoints: doc.endpoints?.map((ep: any) => ({
          method: ep.method || 'GET',
          path: ep.path || '/',
          description: ep.description || '',
        })) || [],
      };

      setServices(prevServices => [...prevServices, newService]);
      toast({
          title: 'Service Created',
          description: `${newService.name} has been added to your list.`,
      });
    } catch (e) {
      console.error("YAML parsing error:", e);
      toast({
        variant: 'destructive',
        title: 'Invalid YAML',
        description: 'Could not parse the service definition. Please check the syntax.',
      });
    }
  }, [toast]);

  /**
   * Updates an existing service in the list.
   * @param {Service} updatedService - The service object with updated information.
   */
  const handleUpdateService = useCallback(async (updatedService: Service) => {
    try {
      const yamlModule = await import('js-yaml');
      const doc: any = yamlModule.default.load(updatedService.definition);
      
      setServices(prevServices => prevServices.map(service => {
        if (service.id === updatedService.id) {
            return {
                ...updatedService,
                name: doc.name || updatedService.name,
                version: doc.version || updatedService.version,
                port: doc.server?.port || updatedService.port,
                endpoints: doc.endpoints?.map((ep: any) => ({
                  method: ep.method || 'GET',
                  path: ep.path || '/',
                  description: ep.description || '',
                })) || [],
            }
        }
        return service;
      }));

      toast({
        title: 'Service Updated',
        description: `${updatedService.name} has been successfully updated.`,
      });
    } catch (e) {
      console.error("YAML parsing error:", e);
      toast({
        variant: 'destructive',
        title: 'Invalid YAML',
        description: 'Could not parse the service definition. Please check the syntax.',
      });
    }
  }, [services, toast]);

  /**
   * Deletes a service from the list.
   * @param {string} serviceId - The ID of the service to delete.
   */
  const handleDeleteService = useCallback((serviceId: string) => {
    const serviceToDelete = services.find(s => s.id === serviceId);
    if (serviceToDelete) {
      setServices(prevServices => prevServices.filter(service => service.id !== serviceId));
      toast({
        title: 'Service Deleted',
        description: `${serviceToDelete.name} has been removed.`,
      });
    }
  }, [services, toast]);

  /**
   * Renders the component for the currently active view.
   * @returns {React.ReactElement} The component to render.
   */
  const renderContent = useMemo(() => {
    if (isLoading && !simulatorStatus) {
      return (
        <div className="flex items-center justify-center h-full">
          <Loader2 className="h-8 w-8 animate-spin text-primary" />
        </div>
      );
    }

    if (error && !simulatorStatus) {
      return (
        <BackendConnectionError
          error={error}
          onRetry={() => refetch()}
        />
      );
    }
    
    switch (activeView) {
      case 'dashboard':
        return <Dashboard services={services} onToggleService={handleToggleService} />;
      case 'services':
        return (
          <Suspense fallback={<ComponentLoader />}>
            <ServiceManagement 
              services={services} 
              onAddService={handleAddService} 
              onUpdateService={handleUpdateService}
              onDeleteService={handleDeleteService}
            />
          </Suspense>
        );
      case 'logs':
        return (
          <Suspense fallback={<ComponentLoader />}>
            <LogsViewer services={simulatorStatus?.active_services || []} />
          </Suspense>
        );
      case 'ai-generator':
        return (
          <Suspense fallback={<ComponentLoader />}>
            <AiGenerator onAddService={handleAddService} />
          </Suspense>
        );
      case 'plugin-generator':
        return (
          <Suspense fallback={<ComponentLoader />}>
            <PluginGenerator />
          </Suspense>
        );
      case 'code-generator':
        return (
          <Suspense fallback={<ComponentLoader />}>
            <CodeGenerator services={services} isLoading={isLoading} />
          </Suspense>
        );
      case 'contract-testing':
        return (
          <Suspense fallback={<ComponentLoader />}>
            <ContractTesting services={services} />
          </Suspense>
        );
      case 'recording':
        return (
          <Suspense fallback={<ComponentLoader />}>
            <Recording />
          </Suspense>
        );
      case 'configuration':
        return (
          <Suspense fallback={<ComponentLoader />}>
            <Configuration />
          </Suspense>
        );
      default:
        return <Dashboard services={services} onToggleService={handleToggleService} />;
    }
  }, [activeView, services, handleToggleService, handleAddService, handleUpdateService, handleDeleteService, isLoading, error, simulatorStatus, refetch]);

  /**
   * A map of view names to their corresponding display titles.
   * @const
   */
  const viewTitles: Record<View, string> = {
    dashboard: 'Dashboard',
    services: 'Service Definitions',
    recording: 'Recording',
    'ai-generator': 'AI Service Generator',
    'plugin-generator': 'Plugin Generator',
    'contract-testing': 'Contract Testing',
    'code-generator': 'Client Code Generator',
    logs: 'Simulator Logs',
    configuration: 'Configuration',
  };

  return (
    <MainLayout
      activeView={activeView}
      setActiveView={setActiveView}
      title={viewTitles[activeView]}
      isSimulatorRunning={simulatorStatus?.is_active ?? false}
      onToggleAllServices={handleToggleAllServices}
    >
      {renderContent}
    </MainLayout>
  );
}

// Memoize AppContent to prevent unnecessary re-renders
const MemoizedAppContent = memo(AppContent);

/**
 * The root component of the application.
 * It sets up the React Query client and renders the main application content.
 * @returns {React.ReactElement} The rendered Home page.
 */
export default function Home() {
  return (
    <QueryClientProvider client={queryClient}>
      <MemoizedAppContent />
    </QueryClientProvider>
  )
}
