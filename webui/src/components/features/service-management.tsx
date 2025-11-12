
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
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu';
import { MoreHorizontal, FilePlus, Upload, Download, CheckCircle, XCircle, Box, Pencil, ShieldCheck, Asterisk } from 'lucide-react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { CreateServiceDialog } from './create-service-dialog';
import { CreateGraphQLServiceDialog } from './create-graphql-service-dialog';
import { EditServiceDialog } from './edit-service-dialog';
import type { Service } from '@/lib/types';
import { useToast } from '@/hooks/use-toast';
import { validateService } from '@/services/api';

/**
 * @fileoverview Manages the display and interaction with service definitions,
 * including creation, editing, validation, and deletion of services.
 */

type ServiceManagementProps = {
    services: Service[];
    onAddService: (service: any) => void;
    onUpdateService: (service: Service) => void;
    onDeleteService: (serviceId: string) => void;
}

/**
 * A component for managing API service definitions.
 * @param {ServiceManagementProps} props - The component props.
 * @returns {React.ReactElement} The rendered ServiceManagement component.
 */
export function ServiceManagement({ services, onAddService, onUpdateService, onDeleteService }: ServiceManagementProps) {
  const [editingService, setEditingService] = React.useState<Service | null>(null);
  const { toast } = useToast();

  /**
   * Validates the selected service's definition.
   * @param {Service} service - The service to validate.
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
   * @param {any} serviceData - The data for the new service.
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
   * @param {Service} updatedService - The updated service data.
   */
  const handleUpdateService = (updatedService: Service) => {
    onUpdateService(updatedService);
    toast({
        title: 'Service Updated',
        description: `${updatedService.name} has been successfully updated.`,
    });
  };

  /**
   * Handles deleting a service and displays a toast notification.
   * @param {string} serviceId - The ID of the service to delete.
   */
  const handleDeleteService = (serviceId: string) => {
    const serviceName = services.find(s => s.id === serviceId)?.name || 'The service';
    onDeleteService(serviceId);
    toast({
        title: 'Service Deleted',
        description: `${serviceName} has been removed.`,
    });
  };
  
  return (
    <>
    <Card>
      <CardHeader className="flex flex-row items-center justify-between">
        <div>
            <CardTitle>Service Definitions</CardTitle>
            <CardDescription>Manage, validate, and edit your service definitions.</CardDescription>
        </div>
        <div className="flex gap-2">
            <Button variant="outline"><Upload className="mr-2 h-4 w-4" /> Import</Button>
            <CreateGraphQLServiceDialog onAddService={handleAddService}>
              <Button variant="outline"><Asterisk className="mr-2 h-4 w-4" /> New GraphQL</Button>
            </CreateGraphQLServiceDialog>
            <CreateServiceDialog onAddService={handleAddService}>
              <Button><FilePlus className="mr-2 h-4 w-4" /> New REST</Button>
            </CreateServiceDialog>
        </div>
      </CardHeader>
      <CardContent>
            <div className="rounded-md border">
                <Table>
                <TableHeader>
                    <TableRow>
                    <TableHead>Service Name</TableHead>
                    <TableHead>Status</TableHead>
                    <TableHead>Version</TableHead>
                    <TableHead>Port</TableHead>
                    <TableHead>Endpoints</TableHead>
                    <TableHead className="text-right">Actions</TableHead>
                    </TableRow>
                </TableHeader>
                <TableBody>
                    {services.map((service) => (
                    <TableRow key={service.id}>
                        <TableCell className="font-medium">{service.name}</TableCell>
                        <TableCell>
                            <Badge variant={service.status === 'running' ? 'default' : 'destructive'} className={`${service.status === 'running' ? 'bg-green-500/20 text-green-400 border-green-500/30' : ''}`}>
                                {service.status === 'running' ? <CheckCircle className="mr-1 h-3 w-3" /> : <XCircle className="mr-1 h-3 w-3" />}
                                {service.status}
                            </Badge>
                        </TableCell>
                        <TableCell>{service.version}</TableCell>
                        <TableCell className="font-mono">{service.port}</TableCell>
                        <TableCell>{service.endpoints.length}</TableCell>
                        <TableCell className="text-right">
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
                                onClick={() => handleDeleteService(service.id)}
                            >
                                Delete
                            </DropdownMenuItem>
                            </DropdownMenuContent>
                        </DropdownMenu>
                        </TableCell>
                    </TableRow>
                    ))}
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
    </>
  );
}
