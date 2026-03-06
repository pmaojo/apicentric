'use client';

import * as React from 'react';
import {
  TableCell,
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
  MoreHorizontal,
  Download,
  CheckCircle,
  XCircle,
  Box,
  Pencil,
  ShieldCheck,
  Play,
  Square,
  Trash2,
  Loader2,
} from 'lucide-react';
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from '@/components/ui/tooltip';
import type { Service } from '@/lib/types';

export interface ServiceRowProps {
  service: Service;
  isLoading: boolean;
  isSelected: boolean;
  onSelect: (id: string, checked: boolean) => void;
  onStop: (service: Service) => void;
  onStart: (service: Service) => void;
  onEdit: (service: Service) => void;
  onValidate: (service: Service) => void;
  onDelete: (service: Service) => void;
}

export const ServiceRow = React.memo(function ServiceRow({
  service,
  isLoading,
  isSelected,
  onSelect,
  onStop,
  onStart,
  onEdit,
  onValidate,
  onDelete,
}: ServiceRowProps) {
  return (
    <TableRow data-testid={`service-${service.name}`}>
      <TableCell>
        <Checkbox
          checked={isSelected}
          onCheckedChange={(checked) => onSelect(service.id, checked as boolean)}
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
                  onClick={() => onStop(service)}
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
                  onClick={() => onStart(service)}
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
              <DropdownMenuItem onClick={() => onEdit(service)}>
                <Pencil className="mr-2 h-4 w-4" />
                Edit
              </DropdownMenuItem>
              <DropdownMenuItem onClick={() => onValidate(service)}>
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
                onClick={() => onDelete(service)}
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
});
