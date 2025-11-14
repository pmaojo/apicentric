/**
 * Service Store - Clean Architecture with Zustand
 * 
 * Combines server state (React Query) with client state (Zustand)
 * for a clean separation of concerns.
 */

import * as React from 'react';
import { create } from 'zustand';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { Service } from '@/lib/types';
import { useToast } from '@/hooks/use-toast';
import { container } from '@/container/di-container';

// ============================================================================
// Client State with Zustand
// ============================================================================

interface ServiceStoreState {
  selectedServices: Set<string>;
  isCreating: boolean;
  isUpdating: boolean;
  selectService: (id: string) => void;
  unselectService: (id: string) => void;
  clearSelection: () => void;
  setCreating: (creating: boolean) => void;
  setUpdating: (updating: boolean) => void;
}

export const useServiceStore = create<ServiceStoreState>((set) => ({
  selectedServices: new Set<string>(),
  isCreating: false,
  isUpdating: false,
  
  selectService: (id: string) => set((state: ServiceStoreState) => ({
    selectedServices: new Set(state.selectedServices).add(id)
  })),
  
  unselectService: (id: string) => set((state: ServiceStoreState) => {
    const newSet = new Set(state.selectedServices);
    newSet.delete(id);
    return { selectedServices: newSet };
  }),
  
  clearSelection: () => set({ selectedServices: new Set<string>() }),
  setCreating: (creating: boolean) => set({ isCreating: creating }),
  setUpdating: (updating: boolean) => set({ isUpdating: updating }),
}))

// ============================================================================
// Server State Hooks (React Query)
// ============================================================================

export function useServices() {
  return useQuery({
    queryKey: ['services'],
    queryFn: () => container.serviceManager.listServices(),
    staleTime: 30 * 1000, // 30 seconds
    refetchInterval: 60 * 1000, // 1 minute
  });
}

export function useService(name: string) {
  return useQuery({
    queryKey: ['service', name],
    queryFn: () => container.serviceManager.getService(name),
    enabled: !!name,
  });
}

// ============================================================================
// Mutation Hooks with Business Logic Integration
// ============================================================================

export function useCreateService() {
  const queryClient = useQueryClient();
  const { toast } = useToast();
  const { setCreating } = useServiceStore();

  return useMutation({
    mutationFn: ({ yaml, filename }: { yaml: string; filename?: string }) =>
      container.serviceManager.createService(yaml, filename),
      
    onMutate: () => {
      setCreating(true);
    },
    
    onSuccess: (service: Service) => {
      queryClient.invalidateQueries({ queryKey: ['services'] });
      
      toast({
        title: 'Service Created',
        description: `Service "${service.name}" has been created successfully.`,
      });
      
      setCreating(false);
    },
    
    onError: (error: Error) => {
      toast({
        variant: 'destructive',
        title: 'Failed to Create Service',
        description: error.message,
      });
      
      setCreating(false);
    },
  });
}

export function useUpdateService() {
  const queryClient = useQueryClient();
  const { toast } = useToast();
  const { setUpdating } = useServiceStore();

  return useMutation({
    mutationFn: ({ name, yaml }: { name: string; yaml: string }) =>
      container.serviceManager.updateService(name, yaml),
      
    onMutate: () => {
      setUpdating(true);
    },
    
    onSuccess: (service: Service) => {
      queryClient.setQueryData(['service', service.name], service);
      queryClient.invalidateQueries({ queryKey: ['services'] });
      
      toast({
        title: 'Service Updated',
        description: `Service "${service.name}" has been updated successfully.`,
      });
      
      setUpdating(false);
    },
    
    onError: (error: Error) => {
      toast({
        variant: 'destructive',
        title: 'Failed to Update Service',
        description: error.message,
      });
      
      setUpdating(false);
    },
  });
}

export function useDeleteService() {
  const queryClient = useQueryClient();
  const { toast } = useToast();

  return useMutation({
    mutationFn: (name: string) => container.serviceManager.deleteService(name),
    
    onSuccess: (_, name) => {
      queryClient.removeQueries({ queryKey: ['service', name] });
      queryClient.invalidateQueries({ queryKey: ['services'] });
      
      toast({
        title: 'Service Deleted',
        description: `Service "${name}" has been deleted successfully.`,
        variant: 'destructive',
      });
    },
    
    onError: (error: Error) => {
      toast({
        variant: 'destructive',
        title: 'Failed to Delete Service',
        description: error.message,
      });
    },
  });
}

export function useStartService() {
  const queryClient = useQueryClient();
  const { toast } = useToast();

  return useMutation({
    mutationFn: (name: string) => container.serviceManager.startService(name),
    
    onSuccess: (service: Service) => {
      queryClient.setQueryData(['service', service.name], service);
      queryClient.invalidateQueries({ queryKey: ['services'] });
      
      toast({
        title: 'Service Started',
        description: `Service "${service.name}" is now running on port ${service.port}.`,
      });
    },
    
    onError: (error: Error) => {
      toast({
        variant: 'destructive',
        title: 'Failed to Start Service',
        description: error.message,
      });
    },
  });
}

export function useStopService() {
  const queryClient = useQueryClient();
  const { toast } = useToast();

  return useMutation({
    mutationFn: (name: string) => container.serviceManager.stopService(name),
    
    onSuccess: (service: Service) => {
      queryClient.setQueryData(['service', service.name], service);
      queryClient.invalidateQueries({ queryKey: ['services'] });
      
      toast({
        title: 'Service Stopped',
        description: `Service "${service.name}" has been stopped.`,
        variant: 'destructive',
      });
    },
    
    onError: (error: Error) => {
      toast({
        variant: 'destructive',
        title: 'Failed to Stop Service',
        description: error.message,
      });
    },
  });
}

// ============================================================================
// Real-time Updates Integration
// ============================================================================

export function useServiceUpdates() {
  const queryClient = useQueryClient();

  React.useEffect(() => {
    const unsubscribe = container.webSocketManager.subscribe('service_status', (update: any) => {
      // Update specific service in cache
      queryClient.setQueryData(
        ['service', update.service_name],
        (oldService: Service | undefined) => {
          if (!oldService) return oldService;
          
          return {
            ...oldService,
            status: update.status,
            port: update.port,
          };
        }
      );

      // Also invalidate the list to ensure consistency
      queryClient.invalidateQueries({ queryKey: ['services'] });
    });

    return unsubscribe;
  }, [queryClient]);
}