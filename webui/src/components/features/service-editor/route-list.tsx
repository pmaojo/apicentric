'use client';

import * as React from 'react';
import { Button } from '@/components/ui/button';
import { Plus, Trash2, Copy, Search } from 'lucide-react';
import { Input } from '@/components/ui/input';
import { Badge } from '@/components/ui/badge';
import { RouteConfig } from './layout';
import { cn } from '@/lib/utils';
import { ScrollArea } from '@/components/ui/scroll-area';

interface RouteListProps {
  routes: RouteConfig[];
  selectedId: string | null;
  onSelect: (id: string) => void;
  onAdd: () => void;
  onDelete: (id: string) => void;
}

export function RouteList({ routes, selectedId, onSelect, onAdd, onDelete }: RouteListProps) {
  const [searchTerm, setSearchTerm] = React.useState('');

  const filteredRoutes = routes.filter(r => 
    r.path.toLowerCase().includes(searchTerm.toLowerCase()) || 
    r.method.toLowerCase().includes(searchTerm.toLowerCase())
  );

  const getMethodColor = (method: string) => {
    switch (method.toUpperCase()) {
      case 'GET': return 'text-blue-500';
      case 'POST': return 'text-green-500';
      case 'PUT': return 'text-orange-500';
      case 'DELETE': return 'text-red-500';
      default: return 'text-foreground';
    }
  };

  return (
    <div className="flex flex-col h-full">
      <div className="p-4 border-b space-y-4">
        <div className="flex items-center justify-between">
            <h2 className="font-semibold text-sm text-muted-foreground">ROUTES</h2>
            <Button size="icon" variant="ghost" className="h-6 w-6" onClick={onAdd}>
                <Plus className="h-4 w-4" />
            </Button>
        </div>
        <div className="relative">
          <Search className="absolute left-2 top-2.5 h-4 w-4 text-muted-foreground" />
          <Input 
            placeholder="Filter routes..." 
            className="pl-8" 
            value={searchTerm}
            onChange={(e) => setSearchTerm(e.target.value)}
          />
        </div>
      </div>
      
      <ScrollArea className="flex-1">
        <div className="p-2 space-y-1">
          {filteredRoutes.map(route => (
            <div
              key={route.id}
              className={cn(
                "group flex items-center justify-between p-3 rounded-md cursor-pointer transition-colors text-sm",
                selectedId === route.id ? "bg-accent text-accent-foreground" : "hover:bg-muted"
              )}
              onClick={() => onSelect(route.id)}
            >
              <div className="flex items-center gap-3 overflow-hidden">
                <span className={cn("font-bold w-12 text-xs", getMethodColor(route.method))}>
                  {route.method}
                </span>
                <span className="truncate font-mono opacity-90">{route.path}</span>
              </div>
              <Button
                size="icon"
                variant="ghost"
                className="h-6 w-6 opacity-0 group-hover:opacity-100 transition-opacity text-muted-foreground hover:text-destructive"
                onClick={(e) => {
                  e.stopPropagation();
                  onDelete(route.id);
                }}
              >
                <Trash2 className="h-3 w-3" />
              </Button>
            </div>
          ))}
          
          {filteredRoutes.length === 0 && (
            <div className="text-center py-8 text-muted-foreground text-xs">
              No routes found.
            </div>
          )}
        </div>
      </ScrollArea>
    </div>
  );
}
