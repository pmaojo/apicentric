'use client';

import { useState, useEffect } from 'react';
import type { ApiService, View, Service, SimulatorStatus } from '@/lib/types';
import { MainLayout } from '@/components/layout/main-layout';
import { Dashboard } from '@/components/features/dashboard';
import { ServiceManagement } from '@/components/features/service-management';
import { LogsViewer } from '@/components/features/logs-viewer';
import { AiGenerator } from '@/components/features/ai-generator';
import { PluginGenerator } from '@/components/features/plugin-generator';
import { CodeGenerator } from '@/components/features/code-generator';
import { ContractTesting } from '@/components/features/contract-testing';
import { Recording } from '@/components/features/recording';
import { QueryClient, QueryClientProvider, useQuery, useMutation } from '@tanstack/react-query';
import { fetchSimulatorStatus, startSimulator, stopSimulator, createGraphQLService, validateService } from '@/services/api';
import { Loader2 } from 'lucide-react';
import { useToast } from '@/hooks/use-toast';
import yaml from 'js-yaml';

/**
 * @fileoverview The main entry point component for the Apicentric UI.
 * It manages the overall layout, state, and view switching for the application.
 */

const queryClient = new QueryClient();

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
    refetchInterval: 5000, // Refetch every 5 seconds
  });
  const [services, setServices] = useState<Service[]>([]);
  const { toast } = useToast();

  useEffect(() => {
    if (simulatorStatus) {
      const mappedServices = simulatorStatus.active_services.map((apiService, index) => ({
        id: apiService.id || `service-${index}-${Date.now()}`,
        name: apiService.name,
        status: apiService.is_running ? 'running' : 'stopped',
        port: apiService.port,
        version: apiService.version || '1.0.0',
        definition: apiService.definition || '',
        endpoints: apiService.endpoints?.map(ep => ({
          ...ep,
          method: ep.method as 'GET' | 'POST' | 'PUT' | 'DELETE' | 'PATCH'
        })) || [],
      }));
      setServices(mappedServices);
    }
  }, [simulatorStatus]);

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

  const handleToggleAllServices = () => {
    if (simulatorStatus?.is_active) {
      stopMutation.mutate();
    } else {
      startMutation.mutate();
    }
  };
  
  const handleToggleService = (serviceId: string, status: 'running' | 'stopped') => {
    // This would ideally be a per-service start/stop endpoint
    // For now, we just toggle the whole simulator
    handleToggleAllServices();
  };

  /**
   * Adds a new service to the list after parsing its YAML definition.
   * @param {object} serviceData - The data for the new service from the creation form.
   * @param {string} serviceData.name - The name of the service.
   * @param {string} serviceData.version - The version of the service.
   * @param {number} serviceData.port - The port for the service.
   * @param {string} serviceData.definition - The raw YAML definition string.
   */
  const handleAddService = (serviceData: { name: string, version: string, port: number, definition: string }) => {
    try {
      const doc: any = yaml.load(serviceData.definition);
      
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
  };

  /**
   * Updates an existing service in the list.
   * @param {Service} updatedService - The service object with updated information.
   */
  const handleUpdateService = (updatedService: Service) => {
    try {
      const doc: any = yaml.load(updatedService.definition);
      
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
  };

  /**
   * Deletes a service from the list.
   * @param {string} serviceId - The ID of the service to delete.
   */
  const handleDeleteService = (serviceId: string) => {
    const serviceToDelete = services.find(s => s.id === serviceId);
    if (serviceToDelete) {
      setServices(prevServices => prevServices.filter(service => service.id !== serviceId));
      toast({
        title: 'Service Deleted',
        description: `${serviceToDelete.name} has been removed.`,
      });
    }
  };

  /**
   * Renders the component for the currently active view.
   * @returns {React.ReactElement} The component to render.
   */
  const renderContent = () => {
    if (isLoading && !simulatorStatus) {
      return (
        <div className="flex items-center justify-center h-full">
          <Loader2 className="h-8 w-8 animate-spin text-primary" />
        </div>
      );
    }

    if (error && !simulatorStatus) {
      return (
        <div className="flex items-center justify-center h-full text-destructive">
          <p>Error loading simulator status: {error.message}</p>
        </div>
      );
    }
    
    switch (activeView) {
      case 'dashboard':
        return <Dashboard services={services} onToggleService={handleToggleService} />;
      case 'services':
        return <ServiceManagement 
                  services={services} 
                  onAddService={handleAddService} 
                  onUpdateService={handleUpdateService}
                  onDeleteService={handleDeleteService}
                />;
      case 'logs':
        return <LogsViewer services={services} />;
      case 'ai-generator':
        return <AiGenerator onAddService={handleAddService} />;
      case 'plugin-generator':
        return <PluginGenerator />;
      case 'code-generator':
        return <CodeGenerator services={services} isLoading={isLoading} />;
      case 'contract-testing':
        return <ContractTesting services={services} />;
      case 'recording':
        return <Recording />;
      default:
        return <Dashboard services={services} onToggleService={handleToggleService} />;
    }
  };

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
  };

  return (
    <MainLayout
      activeView={activeView}
      setActiveView={setActiveView}
      title={viewTitles[activeView]}
      isSimulatorRunning={simulatorStatus?.is_active ?? false}
      onToggleAllServices={handleToggleAllServices}
    >
      {renderContent()}
    </MainLayout>
  );
}

/**
 * The root component of the application.
 * It sets up the React Query client and renders the main application content.
 * @returns {React.ReactElement} The rendered Home page.
 */
export default function Home() {
  return (
    <QueryClientProvider client={queryClient}>
      <AppContent />
    </QueryClientProvider>
  )
}
