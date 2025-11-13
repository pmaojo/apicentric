/**
 * Simplified Service Store with Zustand
 */

import { create } from 'zustand';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { useToast } from '@/hooks/use-toast';
import { Service } from '@/lib/types';
import * as api from '@/services/api';

// ============================================================================
// Zustand Store for Client State
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
  
  selectService: (id: string) => set((state) => ({
    selectedServices: new Set(state.selectedServices).add(id)
  })),
  
  unselectService: (id: string) => set((state) => {
    const newSet = new Set(state.selectedServices);
    newSet.delete(id);
    return { selectedServices: newSet };
  }),
  
  clearSelection: () => set({ selectedServices: new Set<string>() }),
  setCreating: (creating: boolean) => set({ isCreating: creating }),
  setUpdating: (updating: boolean) => set({ isUpdating: updating }),
}));

// ============================================================================
// Server State Hooks (React Query)
// ============================================================================

export function useServices() {
  return useQuery({
    queryKey: ['services'],
    queryFn: api.getServices,
    staleTime: 30 * 1000, // 30 seconds
    refetchInterval: 60 * 1000, // 1 minute
  });
}

export function useService(name: string) {
  return useQuery({
    queryKey: ['service', name],
    queryFn: () => api.getService(name),
    enabled: !!name,
  });
}

// ============================================================================
// Mutation Hooks
// ============================================================================

export function useStartService() {
  const queryClient = useQueryClient();
  const { toast } = useToast();

  return useMutation({
    mutationFn: (name: string) => api.startService(name),
    
    onSuccess: (_, name) => {
      // Optimistically update service status
      queryClient.setQueryData(['service', name], (old: Service | undefined) => 
        old ? { ...old, status: 'running' as const } : old
      );
      
      // Invalidate services list to refresh
      queryClient.invalidateQueries({ queryKey: ['services'] });
      
      toast({
        title: 'Service Started',
        description: `Service "${name}" is now running.`,
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
    mutationFn: (name: string) => api.stopService(name),
    
    onSuccess: (_, name) => {
      // Optimistically update service status
      queryClient.setQueryData(['service', name], (old: Service | undefined) => 
        old ? { ...old, status: 'stopped' as const } : old
      );
      
      // Invalidate services list to refresh
      queryClient.invalidateQueries({ queryKey: ['services'] });
      
      toast({
        title: 'Service Stopped',
        description: `Service "${name}" has been stopped.`,
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

export function useCreateService() {
  const queryClient = useQueryClient();
  const { toast } = useToast();
  const { setCreating } = useServiceStore();

  return useMutation({
    mutationFn: ({ yaml, filename }: { yaml: string; filename?: string }) =>
      api.createService(yaml, filename),
      
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
      api.updateService(name, yaml),
      
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
    mutationFn: (name: string) => api.deleteService(name),
    
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

// ============================================================================
// Real-time Updates (Placeholder for now)
// ============================================================================

export function useServiceUpdates() {
  // For now, just return empty - we can integrate WebSocket later
  return;
}