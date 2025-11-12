
'use client';

import * as React from 'react';
import { Input } from '@/components/ui/input';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table';
import { Badge } from '@/components/ui/badge';
import { logs as allLogs } from '@/lib/data';
import { Button } from '@/components/ui/button';
import { Search } from 'lucide-react';
import { Card, CardContent, CardHeader, CardTitle } from '../ui/card';
import { ApiService } from '@/lib/types';

const LOGS_PER_PAGE = 10;

type LogsViewerProps = {
    services: ApiService[];
};

export function LogsViewer({ services }: LogsViewerProps) {
  const [logs, setLogs] = React.useState(allLogs);
  const [currentPage, setCurrentPage] = React.useState(1);

  const [filters, setFilters] = React.useState({
    service: 'all',
    method: 'all',
    status: 'all',
    search: '',
  });

  React.useEffect(() => {
    const filtered = allLogs.filter((log) => {
      const statusClass = Math.floor(log.status / 100);
      const statusFilterMatch =
        filters.status === 'all' ||
        (filters.status === '2xx' && statusClass === 2) ||
        (filters.status === '3xx' && statusClass === 3) ||
        (filters.status === '4xx' && statusClass === 4) ||
        (filters.status === '5xx' && statusClass === 5);

      const searchMatch =
        filters.search === '' ||
        log.route.toLowerCase().includes(filters.search.toLowerCase()) ||
        log.ip.includes(filters.search);

      return (
        (filters.service === 'all' || log.service === filters.service) &&
        (filters.method === 'all' || log.method === filters.method) &&
        statusFilterMatch &&
        searchMatch
      );
    });
    setLogs(filtered);
    setCurrentPage(1);
  }, [filters]);

  const handleFilterChange = (filterName: string, value: string) => {
    setFilters((prev) => ({ ...prev, [filterName]: value }));
  };
  
  const getStatusColor = (status: number) => {
    if (status >= 500) return 'bg-red-500/20 text-red-400 border-red-500/30';
    if (status >= 400) return 'bg-yellow-500/20 text-yellow-400 border-yellow-500/30';
    if (status >= 300) return 'bg-blue-500/20 text-blue-400 border-blue-500/30';
    if (status >= 200) return 'bg-green-500/20 text-green-400 border-green-500/30';
    return 'bg-gray-500/20 text-gray-400 border-gray-500/30';
  };
  
  const totalPages = Math.ceil(logs.length / LOGS_PER_PAGE);
  const paginatedLogs = logs.slice(
    (currentPage - 1) * LOGS_PER_PAGE,
    currentPage * LOGS_PER_PAGE
  );

  return (
    <Card>
        <CardHeader>
            <CardTitle>Simulator Logs</CardTitle>
            <div className="flex flex-col gap-4 md:flex-row">
                <div className="relative flex-grow">
                    <Search className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground" />
                    <Input placeholder="Search by route or IP..." className="pl-10" onChange={(e) => handleFilterChange('search', e.target.value)} />
                </div>
                <div className="grid grid-cols-1 sm:grid-cols-3 gap-4">
                    <Select onValueChange={(value) => handleFilterChange('service', value)} defaultValue="all">
                        <SelectTrigger><SelectValue placeholder="Service" /></SelectTrigger>
                        <SelectContent>
                            <SelectItem value="all">All Services</SelectItem>
                            {services.map(s => <SelectItem key={s.name} value={s.name}>{s.name}</SelectItem>)}
                        </SelectContent>
                    </Select>
                    <Select onValueChange={(value) => handleFilterChange('method', value)} defaultValue="all">
                        <SelectTrigger><SelectValue placeholder="Method" /></SelectTrigger>
                        <SelectContent>
                            <SelectItem value="all">All Methods</SelectItem>
                            <SelectItem value="GET">GET</SelectItem>
                            <SelectItem value="POST">POST</SelectItem>
                            <SelectItem value="PUT">PUT</SelectItem>
                            <SelectItem value="DELETE">DELETE</SelectItem>
                        </SelectContent>
                    </Select>
                    <Select onValueChange={(value) => handleFilterChange('status', value)} defaultValue="all">
                        <SelectTrigger><SelectValue placeholder="Status" /></SelectTrigger>
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
        </CardHeader>
        <CardContent>
            <div className="rounded-md border">
                <Table>
                    <TableHeader>
                    <TableRow>
                        <TableHead>Timestamp</TableHead>
                        <TableHead>Service</TableHead>
                        <TableHead>Request</TableHead>
                        <TableHead className="text-right">Status</TableHead>
                    </TableRow>
                    </TableHeader>
                    <TableBody>
                    {paginatedLogs.length > 0 ? paginatedLogs.map((log) => (
                        <TableRow key={log.id}>
                        <TableCell className="text-muted-foreground">{new Date(log.timestamp).toLocaleString()}</TableCell>
                        <TableCell>{log.service}</TableCell>
                        <TableCell>
                            <div className="flex items-center gap-2">
                                <Badge variant="outline" className="w-20 justify-center font-mono">{log.method}</Badge>
                                <span className="font-mono text-sm">{log.route}</span>
                            </div>
                        </TableCell>
                        <TableCell className="text-right">
                            <Badge variant="outline" className={getStatusColor(log.status)}>{log.status}</Badge>
                        </TableCell>
                        </TableRow>
                    )) : (
                        <TableRow>
                            <TableCell colSpan={4} className="h-24 text-center">
                                No logs found matching your criteria.
                            </TableCell>
                        </TableRow>
                    )}
                    </TableBody>
                </Table>
            </div>
            {totalPages > 1 && (
                <div className="flex items-center justify-end space-x-2 py-4">
                    <Button
                    variant="outline"
                    size="sm"
                    onClick={() => setCurrentPage(prev => Math.max(prev - 1, 1))}
                    disabled={currentPage === 1}
                    >
                    Previous
                    </Button>
                    <span className="text-sm text-muted-foreground">
                        Page {currentPage} of {totalPages}
                    </span>
                    <Button
                    variant="outline"
                    size="sm"
                    onClick={() => setCurrentPage(prev => Math.min(prev + 1, totalPages))}
                    disabled={currentPage === totalPages}
                    >
                    Next
                    </Button>
                </div>
            )}
        </CardContent>
    </Card>
  );
}
