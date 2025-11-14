/**
 * Clean Service Management Component
 * 
 * This component ONLY handles:
 * - Rendering UI elements
 * - Handling user interactions
 * - Displaying loading/error states
 * 
 * It does NOT:
 * - Make API calls directly
 * - Handle business logic
 * - Transform data
 */

'use client';

import * as React from 'react';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Checkbox } from '@/components/ui/checkbox';
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table';
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
  Play,
  Square,
  Trash2,
  Loader2,
  CheckCircle,
  XCircle,
} from 'lucide-react';

// Import clean hooks that handle business logic
import {
  useServices,
  useServiceStore,
  useCreateService,
  useUpdateService,
  useDeleteService,
  useStartService,
  useStopService,
  useServiceUpdates,
} from '@/stores/service-store';

import { Service } from '@/lib/types';
import { CreateServiceDialog } from './create-service-dialog';
import { EditServiceDialog } from './edit-service-dialog';

/**
 * Clean Service Management Component
 * 
 * Follows Single Responsibility Principle:
 * - Only handles UI rendering and user interactions
 * - All business logic is in hooks/stores
 * - All API calls are in repositories
 */
export function ServiceManagement() {
  // ============================================================================
  // State Management (via clean hooks)
  // ============================================================================
  
  const { data: services = [], isLoading, error } = useServices();
  const { 
    selectedServices, 
    selectService, 
    unselectService, 
    clearSelection,
    setCreating,
    setUpdating 
  } = useServiceStore();
  
  // Real-time updates
  useServiceUpdates();
  
  // Mutations
  const createService = useCreateService(setCreating);
  const updateService = useUpdateService(setUpdating);
  const deleteService = useDeleteService();
  const startService = useStartService();
  const stopService = useStopService();
  
  // Local UI state
  const [editingService, setEditingService] = React.useState<Service | null>(null);
  const [deletingService, setDeletingService] = React.useState<Service | null>(null);

  // ============================================================================
  // Event Handlers (only UI logic, no business logic)
  // ============================================================================
  
  const handleServiceToggle = (serviceId: string, checked: boolean) => {
    if (checked) {
      selectService(serviceId);
    } else {
      unselectService(serviceId);
    }
  };

  const handleSelectAll = (checked: boolean) => {
    if (checked) {
      services.forEach(service => selectService(service.id));
    } else {
      clearSelection();
    }
  };

  const handleCreateService = (yaml: string) => {
    createService.mutate({ yaml });
  };

  const handleUpdateService = (name: string, yaml: string) => {
    updateService.mutate({ name, yaml });
    setEditingService(null);
  };

  const handleDeleteService = () => {
    if (deletingService) {
      deleteService.mutate(deletingService.name);
      setDeletingService(null);
    }
  };

  const handleStartService = (service: Service) => {
    startService.mutate(service.name);
  };

  const handleStopService = (service: Service) => {
    stopService.mutate(service.name);
  };

  // ============================================================================
  // Computed Values (UI logic only)
  // ============================================================================
  
  const allSelected = services.length > 0 && selectedServices.size === services.length;
  const someSelected = selectedServices.size > 0 && selectedServices.size < services.length;
  const runningServices = services.filter(s => s.status === 'running');
  const stoppedServices = services.filter(s => s.status === 'stopped');

  // ============================================================================
  // Render Methods (pure UI, no logic)
  // ============================================================================
  
  if (error) {
    return (
      <Card>
        <CardContent className="pt-6">
          <div className="text-center text-destructive">
            <p>Error loading services: {error.message}</p>
            <Button variant="outline" className="mt-4" onClick={() => window.location.reload()}>
              Reload Page
            </Button>
          </div>
        </CardContent>
      </Card>
    );
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-3xl font-bold tracking-tight">Service Management</h2>
          <p className="text-muted-foreground">
            Manage your API service definitions and their lifecycle
          </p>
        </div>
        <CreateServiceDialog onCreateService={handleCreateService} />
      </div>

      {/* Services Overview */}
      <div className="grid gap-4 md:grid-cols-3">
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Total Services</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{services.length}</div>
          </CardContent>
        </Card>
        
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Running</CardTitle>
            <CheckCircle className="h-4 w-4 text-green-500" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold text-green-500">{runningServices.length}</div>
          </CardContent>
        </Card>
        
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Stopped</CardTitle>
            <XCircle className="h-4 w-4 text-red-500" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold text-red-500">{stoppedServices.length}</div>
          </CardContent>
        </Card>
      </div>

      {/* Services Table */}
      <Card>
        <CardHeader>
          <CardTitle>Services</CardTitle>
          <CardDescription>
            Manage your API services, start/stop them, and edit their definitions.
          </CardDescription>
        </CardHeader>
        <CardContent>
          {isLoading ? (
            <div className="flex items-center justify-center h-32">
              <Loader2 className="h-6 w-6 animate-spin" />
            </div>
          ) : services.length === 0 ? (
            <div className="text-center py-8">
              <p className="text-muted-foreground mb-4">No services found</p>
              <CreateServiceDialog onCreateService={handleCreateService} />
            </div>
          ) : (
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead className="w-12">
                    <Checkbox
                      checked={allSelected}
                      onCheckedChange={handleSelectAll}
                      ref={(ref) => {
                        if (ref) ref.indeterminate = someSelected;
                      }}
                    />
                  </TableHead>
                  <TableHead>Name</TableHead>
                  <TableHead>Status</TableHead>
                  <TableHead>Port</TableHead>
                  <TableHead>Endpoints</TableHead>
                  <TableHead>Version</TableHead>
                  <TableHead className="text-right">Actions</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {services.map((service) => (
                  <ServiceTableRow
                    key={service.id}
                    service={service}
                    selected={selectedServices.has(service.id)}
                    onToggle={(checked) => handleServiceToggle(service.id, checked)}
                    onEdit={() => setEditingService(service)}
                    onDelete={() => setDeletingService(service)}
                    onStart={() => handleStartService(service)}
                    onStop={() => handleStopService(service)}
                    isStarting={startService.isPending}
                    isStopping={stopService.isPending}
                  />
                ))}
              </TableBody>
            </Table>
          )}
        </CardContent>
      </Card>

      {/* Edit Service Dialog */}
      {editingService && (
        <EditServiceDialog
          service={editingService}
          open={!!editingService}
          onOpenChange={(open) => !open && setEditingService(null)}
          onUpdateService={handleUpdateService}
        />
      )}

      {/* Delete Confirmation Dialog */}
      <AlertDialog open={!!deletingService} onOpenChange={(open) => !open && setDeletingService(null)}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>Delete Service</AlertDialogTitle>
            <AlertDialogDescription>
              Are you sure you want to delete "{deletingService?.name}"? This action cannot be undone.
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
    </div>
  );
}

// ============================================================================
// Service Table Row Component (pure UI component)
// ============================================================================

interface ServiceTableRowProps {
  service: Service;
  selected: boolean;
  onToggle: (checked: boolean) => void;
  onEdit: () => void;
  onDelete: () => void;
  onStart: () => void;
  onStop: () => void;
  isStarting: boolean;
  isStopping: boolean;
}

function ServiceTableRow({
  service,
  selected,
  onToggle,
  onEdit,  
  onDelete,
  onStart,
  onStop,
  isStarting,
  isStopping,
}: ServiceTableRowProps) {
  const isLoading = isStarting || isStopping;

  return (
    <TableRow>
      <TableCell>
        <Checkbox checked={selected} onCheckedChange={onToggle} />
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
      
      <TableCell>{service.port}</TableCell>
      <TableCell>{service.endpoints.length}</TableCell>
      <TableCell>{service.version}</TableCell>
      
      <TableCell className="text-right">
        <div className="flex items-center justify-end gap-2">
          {service.status === 'running' ? (
            <Button
              variant="destructive"
              size="sm"
              onClick={onStop}
              disabled={isLoading}
            >
              {isStopping ? (
                <Loader2 className="mr-2 h-3 w-3 animate-spin" />
              ) : (
                <Square className="mr-2 h-3 w-3" />
              )}
              Stop
            </Button>
          ) : (
            <Button size="sm" onClick={onStart} disabled={isLoading}>
              {isStarting ? (
                <Loader2 className="mr-2 h-3 w-3 animate-spin" />
              ) : (
                <Play className="mr-2 h-3 w-3" />
              )}
              Start
            </Button>
          )}
          
          <DropdownMenu>
            <DropdownMenuTrigger asChild>
              <Button variant="ghost" size="sm">
                <MoreHorizontal className="h-4 w-4" />
              </Button>
            </DropdownMenuTrigger>
            <DropdownMenuContent align="end">
              <DropdownMenuItem onClick={onEdit}>
                Edit Definition
              </DropdownMenuItem>
              <DropdownMenuItem onClick={onDelete} className="text-destructive">
                <Trash2 className="mr-2 h-4 w-4" />
                Delete Service
              </DropdownMenuItem>
            </DropdownMenuContent>
          </DropdownMenu>
        </div>
      </TableCell>
    </TableRow>
  );
}