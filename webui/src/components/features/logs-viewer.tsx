'use client';

import * as React from 'react';
import { useVirtualizer } from '@tanstack/react-virtual';
import { useSearchParams, useRouter } from 'next/navigation';
import { Input } from '@/components/ui/input';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '../ui/card';
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
} from '@/components/ui/dialog';
import { ScrollArea } from '@/components/ui/scroll-area';
import { Separator } from '@/components/ui/separator';
import { Search, Download, Trash2, Eye, Calendar, X } from 'lucide-react';
import { ApiService } from '@/lib/types';
import { useWebSocketSubscription } from '@/providers/websocket-provider';
import { queryLogs, clearLogs, exportLogs, type RequestLogEntry } from '@/services/api';
import { useToast } from '@/hooks/use-toast';
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
  Popover,
  PopoverContent,
  PopoverTrigger,
} from '@/components/ui/popover';
import { Label } from '@/components/ui/label';

type LogsViewerProps = {
  services: ApiService[];
};

const LOGS_PER_PAGE = 100;

export function LogsViewer({ services }: LogsViewerProps) {
  const router = useRouter();
  const searchParams = useSearchParams();
  const [logs, setLogs] = React.useState<RequestLogEntry[]>([]);
  const [currentPage, setCurrentPage] = React.useState(1);
  const [selectedLog, setSelectedLog] = React.useState<RequestLogEntry | null>(null);
  const [showDetailDialog, setShowDetailDialog] = React.useState(false);
  const [showClearDialog, setShowClearDialog] = React.useState(false);
  const [isLoading, setIsLoading] = React.useState(true);
  const { toast } = useToast();

  // Initialize filters from URL params
  const [filters, setFilters] = React.useState({
    service: searchParams.get('service') || 'all',
    method: searchParams.get('method') || 'all',
    status: searchParams.get('status') || 'all',
    search: searchParams.get('search') || '',
    from: searchParams.get('from') || '',
    to: searchParams.get('to') || '',
  });

  // Memoized filtered logs
  // Optimized: Use useMemo to prevent unnecessary re-renders when logs update
  const filteredLogs = React.useMemo(() => {
    if (!logs || !Array.isArray(logs)) return [];
    
    return logs.filter((log) => {
      const statusClass = Math.floor(log.status / 100);
      const statusFilterMatch =
        filters.status === 'all' ||
        (filters.status === '2xx' && statusClass === 2) ||
        (filters.status === '3xx' && statusClass === 3) ||
        (filters.status === '4xx' && statusClass === 4) ||
        (filters.status === '5xx' && statusClass === 5);

      const searchMatch =
        filters.search === '' ||
        log.path.toLowerCase().includes(filters.search.toLowerCase()) ||
        log.service.toLowerCase().includes(filters.search.toLowerCase());

      const serviceMatch = filters.service === 'all' || log.service === filters.service;
      const methodMatch = filters.method === 'all' || log.method === filters.method;

      // Time range filtering
      let timeMatch = true;
      if (filters.from || filters.to) {
        const logTime = new Date(log.timestamp).getTime();
        if (filters.from) {
          const fromTime = new Date(filters.from).getTime();
          timeMatch = timeMatch && logTime >= fromTime;
        }
        if (filters.to) {
          const toTime = new Date(filters.to).getTime();
          timeMatch = timeMatch && logTime <= toTime;
        }
      }

      return serviceMatch && methodMatch && statusFilterMatch && searchMatch && timeMatch;
    });
  }, [logs, filters]);

  // Memoized paginated logs
  const paginatedLogs = React.useMemo(() => {
    const startIndex = (currentPage - 1) * LOGS_PER_PAGE;
    const endIndex = startIndex + LOGS_PER_PAGE;
    return filteredLogs.slice(startIndex, endIndex);
  }, [filteredLogs, currentPage]);

  // Virtual scrolling setup
  const parentRef = React.useRef<HTMLDivElement>(null);
  const virtualizer = useVirtualizer({
    count: paginatedLogs.length,
    getScrollElement: () => parentRef.current,
    estimateSize: () => 60,
    overscan: 10,
  });

  // Reset page when filters change
  React.useEffect(() => {
    setCurrentPage(1);
  }, [filters]);

  // Log buffering to prevent excessive re-renders
  const logBufferRef = React.useRef<RequestLogEntry[]>([]);

  // Subscribe to real-time log updates via WebSocket
  useWebSocketSubscription('request_log', (logEntry: RequestLogEntry) => {
    logBufferRef.current.push(logEntry);
  }, []);

  // Flush log buffer to state periodically
  React.useEffect(() => {
    const interval = setInterval(() => {
      if (logBufferRef.current.length > 0) {
        // Create a copy of the current buffer and clear it immediately
        const logsToAdd = [...logBufferRef.current];
        logBufferRef.current = [];

        setLogs((prev) => {
          const newLogs = [...prev, ...logsToAdd];
          // Keep only last 1000 logs in memory
          return newLogs.slice(-1000);
        });
      }
    }, 200); // Flush every 200ms

    return () => clearInterval(interval);
  }, []);

  // Load initial logs
  React.useEffect(() => {
    const loadLogs = async () => {
      try {
        setIsLoading(true);
        const response = await queryLogs({ limit: 1000 });
        setLogs(response?.logs || []);
      } catch (error) {
        console.error('Failed to load logs:', error);
        toast({
          title: 'Error',
          description: 'Failed to load logs',
          variant: 'destructive',
        });
      } finally {
        setIsLoading(false);
      }
    };

    loadLogs();
  }, [toast]);

  // Update URL params when filters change
  React.useEffect(() => {
    const params = new URLSearchParams();
    
    if (filters.service !== 'all') params.set('service', filters.service);
    if (filters.method !== 'all') params.set('method', filters.method);
    if (filters.status !== 'all') params.set('status', filters.status);
    if (filters.search) params.set('search', filters.search);
    if (filters.from) params.set('from', filters.from);
    if (filters.to) params.set('to', filters.to);

    const queryString = params.toString();
    const newUrl = queryString ? `?${queryString}` : window.location.pathname;
    
    // Update URL without triggering a navigation
    window.history.replaceState({}, '', newUrl);
  }, [filters]);

  const totalPages = Math.ceil(filteredLogs.length / LOGS_PER_PAGE);
  const showPagination = filteredLogs.length > LOGS_PER_PAGE;

  const handleFilterChange = (filterName: string, value: string) => {
    setFilters((prev) => ({ ...prev, [filterName]: value }));
  };

  const clearTimeRange = () => {
    setFilters((prev) => ({ ...prev, from: '', to: '' }));
  };

  const hasActiveFilters = () => {
    return (
      filters.service !== 'all' ||
      filters.method !== 'all' ||
      filters.status !== 'all' ||
      filters.search !== '' ||
      filters.from !== '' ||
      filters.to !== ''
    );
  };

  const clearAllFilters = () => {
    setFilters({
      service: 'all',
      method: 'all',
      status: 'all',
      search: '',
      from: '',
      to: '',
    });
  };

  const handleLogClick = (log: RequestLogEntry) => {
    setSelectedLog(log);
    setShowDetailDialog(true);
  };

  const handleClearLogs = async () => {
    try {
      await clearLogs();
      setLogs([]);
      setShowClearDialog(false);
      toast({
        title: 'Success',
        description: 'Logs cleared successfully',
      });
    } catch (error) {
      console.error('Failed to clear logs:', error);
      toast({
        title: 'Error',
        description: 'Failed to clear logs',
        variant: 'destructive',
      });
    }
  };

  const handleExportLogs = async (format: 'json' | 'csv') => {
    try {
      const data = await exportLogs(format);
      const blob = new Blob([data], {
        type: format === 'json' ? 'application/json' : 'text/csv',
      });
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = `logs-${new Date().toISOString()}.${format}`;
      document.body.appendChild(a);
      a.click();
      document.body.removeChild(a);
      URL.revokeObjectURL(url);
      
      toast({
        title: 'Success',
        description: `Logs exported as ${format.toUpperCase()}`,
      });
    } catch (error) {
      console.error('Failed to export logs:', error);
      toast({
        title: 'Error',
        description: 'Failed to export logs',
        variant: 'destructive',
      });
    }
  };

  const getStatusColor = (status: number) => {
    if (status >= 500) return 'bg-red-500/20 text-red-400 border-red-500/30';
    if (status >= 400) return 'bg-yellow-500/20 text-yellow-400 border-yellow-500/30';
    if (status >= 300) return 'bg-blue-500/20 text-blue-400 border-blue-500/30';
    if (status >= 200) return 'bg-green-500/20 text-green-400 border-green-500/30';
    return 'bg-gray-500/20 text-gray-400 border-gray-500/30';
  };

  const formatTimestamp = (timestamp: string) => {
    return new Date(timestamp).toLocaleString();
  };

  return (
    <>
      <Card>
        <CardHeader>
          <div className="flex items-center justify-between">
            <CardTitle>Simulator Logs</CardTitle>
            <div className="flex gap-2">
              <Button
                variant="outline"
                size="sm"
                onClick={() => handleExportLogs('json')}
                disabled={filteredLogs.length === 0}
              >
                <Download className="h-4 w-4 mr-2" />
                JSON
              </Button>
              <Button
                variant="outline"
                size="sm"
                onClick={() => handleExportLogs('csv')}
                disabled={filteredLogs.length === 0}
              >
                <Download className="h-4 w-4 mr-2" />
                CSV
              </Button>
              <Button
                variant="outline"
                size="sm"
                onClick={() => setShowClearDialog(true)}
                disabled={logs.length === 0}
              >
                <Trash2 className="h-4 w-4 mr-2" />
                Clear
              </Button>
            </div>
          </div>
          <div className="flex flex-col gap-4 md:flex-row mt-4">
            <div className="relative flex-grow">
              <Search className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground" />
              <Input
                placeholder="Search by path or service..."
                className="pl-10"
                value={filters.search}
                onChange={(e) => handleFilterChange('search', e.target.value)}
              />
            </div>
            <div className="grid grid-cols-1 sm:grid-cols-3 gap-4">
              <Select
                onValueChange={(value) => handleFilterChange('service', value)}
                value={filters.service}
              >
                <SelectTrigger>
                  <SelectValue placeholder="Service" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="all">All Services</SelectItem>
                  {services.map((s) => (
                    <SelectItem key={s.name} value={s.name}>
                      {s.name}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
              <Select
                onValueChange={(value) => handleFilterChange('method', value)}
                value={filters.method}
              >
                <SelectTrigger>
                  <SelectValue placeholder="Method" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="all">All Methods</SelectItem>
                  <SelectItem value="GET">GET</SelectItem>
                  <SelectItem value="POST">POST</SelectItem>
                  <SelectItem value="PUT">PUT</SelectItem>
                  <SelectItem value="DELETE">DELETE</SelectItem>
                  <SelectItem value="PATCH">PATCH</SelectItem>
                </SelectContent>
              </Select>
              <Select
                onValueChange={(value) => handleFilterChange('status', value)}
                value={filters.status}
              >
                <SelectTrigger>
                  <SelectValue placeholder="Status" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="all">All Statuses</SelectItem>
                  <SelectItem value="2xx">2xx Success</SelectItem>
                  <SelectItem value="3xx">3xx Redirection</SelectItem>
                  <SelectItem value="4xx">4xx Client Error</SelectItem>
                  <SelectItem value="5xx">5xx Server Error</SelectItem>
                </SelectContent>
              </Select>
            </div>
          </div>
          <div className="flex items-center justify-between mt-2">
            <div className="flex items-center gap-2">
              <Popover>
                <PopoverTrigger asChild>
                  <Button variant="outline" size="sm">
                    <Calendar className="h-4 w-4 mr-2" />
                    Time Range
                    {(filters.from || filters.to) && (
                      <Badge variant="secondary" className="ml-2">
                        Active
                      </Badge>
                    )}
                  </Button>
                </PopoverTrigger>
                <PopoverContent className="w-80">
                  <div className="space-y-4">
                    <div className="space-y-2">
                      <Label htmlFor="from">From</Label>
                      <Input
                        id="from"
                        type="datetime-local"
                        value={filters.from}
                        onChange={(e) => handleFilterChange('from', e.target.value)}
                      />
                    </div>
                    <div className="space-y-2">
                      <Label htmlFor="to">To</Label>
                      <Input
                        id="to"
                        type="datetime-local"
                        value={filters.to}
                        onChange={(e) => handleFilterChange('to', e.target.value)}
                      />
                    </div>
                    {(filters.from || filters.to) && (
                      <Button
                        variant="outline"
                        size="sm"
                        onClick={clearTimeRange}
                        className="w-full"
                      >
                        Clear Time Range
                      </Button>
                    )}
                  </div>
                </PopoverContent>
              </Popover>
              {hasActiveFilters() && (
                <Button
                  variant="ghost"
                  size="sm"
                  onClick={clearAllFilters}
                >
                  <X className="h-4 w-4 mr-2" />
                  Clear All Filters
                </Button>
              )}
            </div>
            <div className="text-sm text-muted-foreground">
              Showing {filteredLogs.length} of {logs.length} logs
            </div>
          </div>
        </CardHeader>
        <CardContent>
          {isLoading ? (
            <div className="h-96 flex items-center justify-center">
              <p className="text-muted-foreground">Loading logs...</p>
            </div>
          ) : filteredLogs.length === 0 ? (
            <div className="h-96 flex items-center justify-center">
              <p className="text-muted-foreground">No logs found matching your criteria.</p>
            </div>
          ) : (
            <>
              <div
                ref={parentRef}
                className="h-[600px] overflow-auto rounded-md border"
              >
                <div
                  style={{
                    height: `${virtualizer.getTotalSize()}px`,
                    width: '100%',
                    position: 'relative',
                  }}
                >
                  {virtualizer.getVirtualItems().map((virtualRow) => {
                    const log = paginatedLogs[virtualRow.index];
                    return (
                      <div
                        key={virtualRow.index}
                        role="button"
                        tabIndex={0}
                        aria-label={`${log.method} request to ${log.path} returned ${log.status}`}
                        style={{
                          position: 'absolute',
                          top: 0,
                          left: 0,
                          width: '100%',
                          height: `${virtualRow.size}px`,
                          transform: `translateY(${virtualRow.start}px)`,
                        }}
                        className="border-b hover:bg-accent/50 cursor-pointer transition-colors focus-visible:outline-none focus-visible:bg-accent focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-inset"
                        onClick={() => handleLogClick(log)}
                        onKeyDown={(e) => {
                          if (e.key === 'Enter' || e.key === ' ') {
                            e.preventDefault();
                            handleLogClick(log);
                          }
                        }}
                      >
                        <div className="flex items-center gap-4 p-4">
                          <div className="flex-shrink-0 w-40 text-xs text-muted-foreground">
                            {formatTimestamp(log.timestamp)}
                          </div>
                          <div className="flex-shrink-0 w-32">
                            <span className="text-sm">{log.service}</span>
                          </div>
                          <div className="flex items-center gap-2 flex-grow min-w-0">
                            <Badge
                              variant="outline"
                              className="w-20 justify-center font-mono flex-shrink-0"
                            >
                              {log.method}
                            </Badge>
                            <span className="font-mono text-sm truncate">
                              {log.path}
                            </span>
                          </div>
                          <div className="flex items-center gap-2 flex-shrink-0">
                            {log.duration_ms && (
                              <span className="text-xs text-muted-foreground">
                                {log.duration_ms}ms
                              </span>
                            )}
                            <Badge
                              variant="outline"
                              className={getStatusColor(log.status)}
                            >
                              {log.status}
                            </Badge>
                            <Eye className="h-4 w-4 text-muted-foreground" />
                          </div>
                        </div>
                      </div>
                    );
                  })}
                </div>
              </div>
              {showPagination && (
                <div className="flex items-center justify-between mt-4">
                  <div className="text-sm text-muted-foreground">
                    Page {currentPage} of {totalPages} ({filteredLogs.length} total logs)
                  </div>
                  <div className="flex items-center gap-2">
                    <Button
                      variant="outline"
                      size="sm"
                      onClick={() => setCurrentPage(1)}
                      disabled={currentPage === 1}
                    >
                      First
                    </Button>
                    <Button
                      variant="outline"
                      size="sm"
                      onClick={() => setCurrentPage((prev) => Math.max(prev - 1, 1))}
                      disabled={currentPage === 1}
                    >
                      Previous
                    </Button>
                    <Button
                      variant="outline"
                      size="sm"
                      onClick={() => setCurrentPage((prev) => Math.min(prev + 1, totalPages))}
                      disabled={currentPage === totalPages}
                    >
                      Next
                    </Button>
                    <Button
                      variant="outline"
                      size="sm"
                      onClick={() => setCurrentPage(totalPages)}
                      disabled={currentPage === totalPages}
                    >
                      Last
                    </Button>
                  </div>
                </div>
              )}
            </>
          )}
        </CardContent>
      </Card>

      {/* Log Detail Dialog */}
      <Dialog open={showDetailDialog} onOpenChange={setShowDetailDialog}>
        <DialogContent className="max-w-4xl max-h-[80vh]">
          <DialogHeader>
            <DialogTitle>Request Details</DialogTitle>
            <DialogDescription>
              {selectedLog && (
                <div className="flex items-center gap-2 mt-2">
                  <Badge variant="outline" className="font-mono">
                    {selectedLog.method}
                  </Badge>
                  <span className="font-mono text-sm">{selectedLog.path}</span>
                  <Badge
                    variant="outline"
                    className={getStatusColor(selectedLog.status)}
                  >
                    {selectedLog.status}
                  </Badge>
                </div>
              )}
            </DialogDescription>
          </DialogHeader>
          {selectedLog && (
            <ScrollArea className="h-[60vh]">
              <div className="space-y-4">
                {/* Metadata */}
                <div>
                  <h3 className="font-semibold mb-2">Metadata</h3>
                  <div className="grid grid-cols-2 gap-2 text-sm">
                    <div>
                      <span className="text-muted-foreground">Timestamp:</span>{' '}
                      {formatTimestamp(selectedLog.timestamp)}
                    </div>
                    <div>
                      <span className="text-muted-foreground">Service:</span>{' '}
                      {selectedLog.service}
                    </div>
                    {selectedLog.duration_ms && (
                      <div>
                        <span className="text-muted-foreground">Duration:</span>{' '}
                        {selectedLog.duration_ms}ms
                      </div>
                    )}
                  </div>
                </div>

                <Separator />

                {/* Request Headers */}
                {selectedLog.request_headers && (
                  <>
                    <div>
                      <h3 className="font-semibold mb-2">Request Headers</h3>
                      <pre className="bg-muted p-3 rounded-md text-xs overflow-x-auto">
                        {JSON.stringify(selectedLog.request_headers, null, 2)}
                      </pre>
                    </div>
                    <Separator />
                  </>
                )}

                {/* Request Body */}
                {selectedLog.request_body && (
                  <>
                    <div>
                      <h3 className="font-semibold mb-2">Request Body</h3>
                      <pre className="bg-muted p-3 rounded-md text-xs overflow-x-auto">
                        {selectedLog.request_body}
                      </pre>
                    </div>
                    <Separator />
                  </>
                )}

                {/* Response Headers */}
                {selectedLog.response_headers && (
                  <>
                    <div>
                      <h3 className="font-semibold mb-2">Response Headers</h3>
                      <pre className="bg-muted p-3 rounded-md text-xs overflow-x-auto">
                        {JSON.stringify(selectedLog.response_headers, null, 2)}
                      </pre>
                    </div>
                    <Separator />
                  </>
                )}

                {/* Response Body */}
                {selectedLog.response_body && (
                  <div>
                    <h3 className="font-semibold mb-2">Response Body</h3>
                    <pre className="bg-muted p-3 rounded-md text-xs overflow-x-auto">
                      {selectedLog.response_body}
                    </pre>
                  </div>
                )}
              </div>
            </ScrollArea>
          )}
        </DialogContent>
      </Dialog>

      {/* Clear Logs Confirmation Dialog */}
      <AlertDialog open={showClearDialog} onOpenChange={setShowClearDialog}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>Clear All Logs?</AlertDialogTitle>
            <AlertDialogDescription>
              This action cannot be undone. This will permanently delete all
              request logs from the system.
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>Cancel</AlertDialogCancel>
            <AlertDialogAction onClick={handleClearLogs}>
              Clear Logs
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </>
  );
}