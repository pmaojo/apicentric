
'use client';

import * as React from 'react';
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Checkbox } from '@/components/ui/checkbox';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu';
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
import {
  MoreHorizontal,
  FilePlus,
  Upload,
  Download,
  CheckCircle,
  XCircle,
  Box,
  Pencil,
  ShieldCheck,
  Asterisk,
  Play,
  Square,
  Trash2,
  Loader2,
} from 'lucide-react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from '@/components/ui/tooltip';
import { CreateServiceDialog } from './create-service-dialog';
import { CreateGraphQLServiceDialog } from './create-graphql-service-dialog';
import { EditServiceDialog } from './edit-service-dialog';
import type { Service } from '@/lib/types';
import { useToast } from '@/hooks/use-toast';
import { useWebSocketSubscription, type ServiceStatusUpdate } from '@/providers/websocket-provider';
import { validateService, startService, stopService, deleteService } from '@/services/api';

/**
 * @fileoverview Manages the display and interaction with service definitions,
 * including creation, editing, validation, and deletion of services.
 */

type ServiceManagementProps = {
    services: Service[];
    onAddService: (service: any) => void;
    onUpdateService: (service: Service) => void;
    onDeleteService: (serviceId: string) => void;
    onServiceUpdate?: (serviceName: string, updates: Partial<Service>) => void;
}

/**
 * A component for managing API service definitions.
 * @param {ServiceManagementProps} props - The component props.
 * @returns {React.ReactElement} The rendered ServiceManagement component.
 */
export function ServiceManagement({
  services,
  onAddService,
  onUpdateService,
  onDeleteService,
  onServiceUpdate,
}: ServiceManagementProps) {
  const [editingService, setEditingService] = React.useState<Service | null>(null);
  const [deletingService, setDeletingService] = React.useState<Service | null>(null);
  const [selectedServices, setSelectedServices] = React.useState<Set<string>>(new Set());
  const [loadingServices, setLoadingServices] = React.useState<Set<string>>(new Set());
  const [bulkOperationLoading, setBulkOperationLoading] = React.useState(false);
  const { toast } = useToast();

  // Subscribe to service status updates via WebSocket
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
  });

  /**
   * Validates the selected service's definition.
   */
  const handleValidate = async (service: Service) => {
    try {
      await validateService(service.definition);
      toast({
        title: 'Validation Successful',
        description: `The definition for "${service.name}" is valid.`,
      });
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'An unknown error occurred.';
      toast({
        variant: 'destructive',
        title: 'Validation Failed',
        description: errorMessage,
      });
    }
  };

  /**
   * Handles adding a new service and displays a toast notification.
   */
  const handleAddService = (serviceData: any) => {
    onAddService(serviceData);
    toast({
        title: 'Service Created',
        description: `${serviceData.name} has been added.`,
    });
  };

  /**
   * Handles updating a service and displays a toast notification.
   */
  const handleUpdateService = (updatedService: Service) => {
    onUpdateService(updatedService);
    toast({
        title: 'Service Updated',
        description: `${updatedService.name} has been successfully updated.`,
    });
  };

  /**
   * Handles starting a service.
   */
  const handleStartService = async (service: Service) => {
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
  };

  /**
   * Handles stopping a service.
   */
  const handleStopService = async (service: Service) => {
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
  };

  /**
   * Handles deleting a service with confirmation.
   */
  const handleDeleteService = async () => {
    if (!deletingService) return;
    
    try {
      await deleteService(deletingService.name);
      onDeleteService(deletingService.id);
      toast({
        title: 'Service Deleted',
        description: `${deletingService.name} has been removed.`,
      });
    } catch (error) {
      toast({
        variant: 'destructive',
        title: 'Failed to Delete Service',
        description: error instanceof Error ? error.message : 'An unknown error occurred',
      });
    } finally {
      setDeletingService(null);
    }
  };

  /**
   * Handles selecting/deselecting all services.
   */
  const handleSelectAll = (checked: boolean) => {
    if (checked) {
      setSelectedServices(new Set(services.map(s => s.id)));
    } else {
      setSelectedServices(new Set());
    }
  };

  /**
   * Handles selecting/deselecting a single service.
   */
  const handleSelectService = (serviceId: string, checked: boolean) => {
    setSelectedServices((prev: Set<string>) => {
      const next = new Set(prev);
      if (checked) {
        next.add(serviceId);
      } else {
        next.delete(serviceId);
      }
      return next;
    });
  };

  /**
   * Handles starting all selected services.
   */
  const handleStartAll = async () => {
    setBulkOperationLoading(true);
    const selectedServicesList = services.filter(s => selectedServices.has(s.id));
    
    try {
      await Promise.all(
        selectedServicesList.map(service => startService(service.name))
      );
      toast({
        title: 'Services Starting',
        description: `Starting ${selectedServicesList.length} service(s)...`,
      });
      setSelectedServices(new Set());
    } catch (error) {
      toast({
        variant: 'destructive',
        title: 'Failed to Start Services',
        description: error instanceof Error ? error.message : 'An unknown error occurred',
      });
    } finally {
      setBulkOperationLoading(false);
    }
  };

  /**
   * Handles stopping all selected services.
   */
  const handleStopAll = async () => {
    setBulkOperationLoading(true);
    const selectedServicesList = services.filter(s => selectedServices.has(s.id));
    
    try {
      await Promise.all(
        selectedServicesList.map(service => stopService(service.name))
      );
      toast({
        title: 'Services Stopping',
        description: `Stopping ${selectedServicesList.length} service(s)...`,
      });
      setSelectedServices(new Set());
    } catch (error) {
      toast({
        variant: 'destructive',
        title: 'Failed to Stop Services',
        description: error instanceof Error ? error.message : 'An unknown error occurred',
      });
    } finally {
      setBulkOperationLoading(false);
    }
  };

  const hasSelection = selectedServices.size > 0;
  const allSelected = services.length > 0 && selectedServices.size === services.length;
  
  return (
    <TooltipProvider>
    <Card>
      <CardHeader className="flex flex-row items-center justify-between">
        <div>
            <CardTitle>Service Definitions</CardTitle>
            <CardDescription>Manage, validate, and edit your service definitions.</CardDescription>
        </div>
        <div className="flex gap-2">
            {hasSelection && (
              <>
                <Button
                  variant="outline"
                  size="sm"
                  onClick={handleStartAll}
                  disabled={bulkOperationLoading}
                >
                  {bulkOperationLoading ? (
                    <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                  ) : (
                    <Play className="mr-2 h-4 w-4" />
                  )}
                  Start Selected ({selectedServices.size})
                </Button>
                <Button
                  variant="outline"
                  size="sm"
                  onClick={handleStopAll}
                  disabled={bulkOperationLoading}
                >
                  {bulkOperationLoading ? (
                    <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                  ) : (
                    <Square className="mr-2 h-4 w-4" />
                  )}
                  Stop Selected ({selectedServices.size})
                </Button>
              </>
            )}
            <Button variant="outline"><Upload className="mr-2 h-4 w-4" /> Import</Button>
            <CreateGraphQLServiceDialog onAddService={handleAddService}>
              <Button variant="outline"><Asterisk className="mr-2 h-4 w-4" /> New GraphQL</Button>
            </CreateGraphQLServiceDialog>
            <CreateServiceDialog onAddService={handleAddService}>
              <Button data-testid="create-service-button"><FilePlus className="mr-2 h-4 w-4" /> New REST</Button>
            </CreateServiceDialog>
        </div>
      </CardHeader>
      <CardContent>
            <div className="rounded-md border">
                <Table>
                <TableHeader>
                    <TableRow>
                    <TableHead className="w-12">
                      <Checkbox
                        checked={allSelected}
                        onCheckedChange={handleSelectAll}
                        aria-label="Select all services"
                      />
                    </TableHead>
                    <TableHead>Service Name</TableHead>
                    <TableHead>Status</TableHead>
                    <TableHead>Version</TableHead>
                    <TableHead>Port</TableHead>
                    <TableHead>Endpoints</TableHead>
                    <TableHead className="text-right">Actions</TableHead>
                    </TableRow>
                </TableHeader>
                <TableBody>
                    {services.length === 0 ? (
                      <TableRow>
                        <TableCell colSpan={7} className="h-48 text-center">
                          <div className="flex flex-col items-center justify-center gap-2">
                            <Box className="h-10 w-10 text-muted-foreground/50" />
                            <h3 className="text-lg font-semibold">No services yet</h3>
                            <p className="text-sm text-muted-foreground mb-4">
                              Create your first service to get started with the simulator.
                            </p>
                            <CreateServiceDialog onAddService={handleAddService}>
                              <Button><FilePlus className="mr-2 h-4 w-4" /> Create New Service</Button>
                            </CreateServiceDialog>
                          </div>
                        </TableCell>
                      </TableRow>
                    ) : (
                      services.map((service) => {
                      const isLoading = loadingServices.has(service.name);
                      const isSelected = selectedServices.has(service.id);
                      
                      return (
                        <TableRow key={service.id} data-testid={`service-${service.name}`}>
                          <TableCell>
                            <Checkbox
                              checked={isSelected}
                              onCheckedChange={(checked) => handleSelectService(service.id, checked as boolean)}
                              aria-label={`Select ${service.name}`}
                            />
                          </TableCell>
                          <TableCell className="font-medium">{service.name}</TableCell>
                          <TableCell>
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
                          </TableCell>
                          <TableCell>{service.version}</TableCell>
                          <TableCell className="font-mono">{service.port}</TableCell>
                          <TableCell>{service.endpoints.length}</TableCell>
                          <TableCell className="text-right">
                          <div className="flex items-center justify-end gap-2">
                            {service.status === 'running' ? (
                              <Tooltip>
                                <TooltipTrigger asChild>
                                  <Button
                                    variant="ghost"
                                    size="sm"
                                    onClick={() => handleStopService(service)}
                                    disabled={isLoading}
                                    data-testid="stop-service-button"
                                    aria-label={`Stop ${service.name}`}
                                  >
                                    {isLoading ? (
                                      <Loader2 className="h-4 w-4 animate-spin" />
                                    ) : (
                                      <Square className="h-4 w-4" />
                                    )}
                                  </Button>
                                </TooltipTrigger>
                                <TooltipContent>
                                  <p>Stop Service</p>
                                </TooltipContent>
                              </Tooltip>
                            ) : (
                              <Tooltip>
                                <TooltipTrigger asChild>
                                  <Button
                                    variant="ghost"
                                    size="sm"
                                    onClick={() => handleStartService(service)}
                                    disabled={isLoading}
                                    data-testid="start-service-button"
                                    aria-label={`Start ${service.name}`}
                                  >
                                    {isLoading ? (
                                      <Loader2 className="h-4 w-4 animate-spin" />
                                    ) : (
                                      <Play className="h-4 w-4" />
                                    )}
                                  </Button>
                                </TooltipTrigger>
                                <TooltipContent>
                                  <p>Start Service</p>
                                </TooltipContent>
                              </Tooltip>
                            )}
                            <DropdownMenu>
                                <DropdownMenuTrigger asChild>
                                <Button variant="ghost" className="h-8 w-8 p-0">
                                    <span className="sr-only">Open menu</span>
                                    <MoreHorizontal className="h-4 w-4" />
                                </Button>
                                </DropdownMenuTrigger>
                                <DropdownMenuContent align="end">
                                <DropdownMenuItem onClick={() => setEditingService(service)}>
                                    <Pencil className="mr-2 h-4 w-4" />
                                    Edit
                                </DropdownMenuItem>
                                <DropdownMenuItem onClick={() => handleValidate(service)}>
                                    <ShieldCheck className="mr-2 h-4 w-4" />
                                    Validate
                                </DropdownMenuItem>
                                <DropdownMenuItem>
                                    <Download className="mr-2 h-4 w-4" />
                                    Export
                                </DropdownMenuItem>
                                <DropdownMenuItem>
                                    <Box className="mr-2 h-4 w-4" />
                                    Dockerize
                                </DropdownMenuItem>
                                <DropdownMenuItem
                                    className="text-destructive"
                                    onClick={() => setDeletingService(service)}
                                >
                                    <Trash2 className="mr-2 h-4 w-4" />
                                    Delete
                                </DropdownMenuItem>
                                </DropdownMenuContent>
                            </DropdownMenu>
                          </div>
                          </TableCell>
                        </TableRow>
                      );
                    }))}
                </TableBody>
                </Table>
            </div>
      </CardContent>
    </Card>
    
    {editingService && (
        <EditServiceDialog
            service={editingService}
            onUpdateService={(updatedService) => {
                handleUpdateService(updatedService);
                setEditingService(null);
            }}
            onOpenChange={(isOpen) => {
                if (!isOpen) {
                    setEditingService(null);
                }
            }}
        />
    )}

    <AlertDialog open={!!deletingService} onOpenChange={(open) => !open && setDeletingService(null)}>
      <AlertDialogContent>
        <AlertDialogHeader>
          <AlertDialogTitle>Delete Service</AlertDialogTitle>
          <AlertDialogDescription>
            Are you sure you want to delete {deletingService?.name}? This action cannot be undone and will
            permanently remove the service definition file.
          </AlertDialogDescription>
        </AlertDialogHeader>
        <AlertDialogFooter>
          <AlertDialogCancel>Cancel</AlertDialogCancel>
          <AlertDialogAction
            onClick={handleDeleteService}
            className="bg-destructive text-destructive-foreground hover:bg-destructive/90"
          >
            Delete Service
          </AlertDialogAction>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
    </TooltipProvider>
  );
}
