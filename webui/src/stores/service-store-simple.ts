/**
 * Service Store - Simple Clean Architecture Hooks
 * 
 * Provides React hooks that encapsulate service operations
 * using the clean architecture pattern.
 */

import * as React from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { useToast } from '@/hooks/use-toast';
import { container } from '@/container/di-container';
import type { Service } from '@/lib/types';

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

export function useStartService() {
  const queryClient = useQueryClient();
  const { toast } = useToast();

  return useMutation({
    mutationFn: (name: string) => container.serviceManager.startService(name),
    
    onSuccess: (service: Service) => {
      // Update cache optimistically
      queryClient.setQueryData(['service', service.name], service);
      
      // Invalidate services list to refresh
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
      // Update cache optimistically
      queryClient.setQueryData(['service', service.name], service);
      
      // Invalidate services list to refresh
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