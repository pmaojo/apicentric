'use client';

import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { RadioTower, PlayCircle, Square, FilePlus } from 'lucide-react';
import { Input } from '@/components/ui/input';
import * as React from 'react';
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '../ui/table';
import { Badge } from '../ui/badge';
import { useToast } from '@/hooks/use-toast';

type CapturedRequest = {
  id: string;
  method: string;
  path: string;
  status: number;
};

const SIMULATED_REQUESTS: Omit<CapturedRequest, 'id'>[] = [
    { method: 'GET', path: '/api/users', status: 200 },
    { method: 'POST', path: '/api/users', status: 201 },
    { method: 'GET', path: '/api/users/123', status: 200 },
    { method: 'PUT', path: '/api/users/123', status: 200 },
    { method: 'GET', path: '/api/products?category=electronics', status: 200 },
    { method: 'GET', path: '/api/products/prod-456', status: 404 },
    { method: 'POST', path: '/api/orders', status: 201 },
];

export function Recording() {
  const [isRecording, setIsRecording] = React.useState(false);
  const [targetUrl, setTargetUrl] = React.useState('https://api.example.com');
  const [capturedRequests, setCapturedRequests] = React.useState<CapturedRequest[]>([]);
  const { toast } = useToast();
  
  React.useEffect(() => {
    let interval: NodeJS.Timeout;
    if (isRecording) {
      interval = setInterval(() => {
        setCapturedRequests(prev => {
          if (prev.length < SIMULATED_REQUESTS.length) {
            const nextRequest = SIMULATED_REQUESTS[prev.length];
            return [...prev, { ...nextRequest, id: `req-${Date.now()}-${prev.length}` }];
          }
          return prev;
        });
      }, 1500);
    }
    return () => clearInterval(interval);
  }, [isRecording]);

  const handleToggleRecording = () => {
    setIsRecording(prev => {
        if (!prev) { // Starting recording
            setCapturedRequests([]); // Clear previous requests
        }
        return !prev;
    });
  };

  const handleGenerateService = () => {
    toast({
        title: "Service Generated",
        description: "A new service definition has been created from the captured requests.",
    });
  };
  
  const getStatusColor = (status: number) => {
    if (status >= 500) return 'bg-red-500/20 text-red-400 border-red-500/30';
    if (status >= 400) return 'bg-yellow-500/20 text-yellow-400 border-yellow-500/30';
    if (status >= 200) return 'bg-green-500/20 text-green-400 border-green-500/30';
    return 'bg-gray-500/20 text-gray-400 border-gray-500/30';
  };

  return (
    <Card>
      <CardHeader>
        <CardTitle>API Traffic Recorder</CardTitle>
        <CardDescription>
          Capture live HTTP traffic and automatically generate service definitions from real API interactions.
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-6">
        <div className="space-y-2">
            <label className="text-sm font-medium">Target URL</label>
            <div className="flex gap-2">
                <Input 
                    placeholder="https://api.example.com" 
                    disabled={isRecording}
                    value={targetUrl}
                    onChange={(e) => setTargetUrl(e.target.value)}
                />
                <Button onClick={handleToggleRecording} disabled={!targetUrl}>
                  {isRecording ? (
                    <>
                      <Square className="mr-2 h-4 w-4" />
                      Stop Recording
                    </>
                  ) : (
                    <>
                      <PlayCircle className="mr-2 h-4 w-4" />
                      Start Recording
                    </>
                  )}
                </Button>
            </div>
            {isRecording && <p className="text-sm text-muted-foreground flex items-center gap-2"><RadioTower className="h-4 w-4 animate-pulse text-primary" />Listening for traffic...</p>}
        </div>
        <div className="space-y-4">
            <div className="flex items-center justify-between">
                <h3 className="text-lg font-semibold">Captured Requests</h3>
                {capturedRequests.length > 0 && (
                    <Button variant="outline" onClick={handleGenerateService}>
                        <FilePlus className="mr-2 h-4 w-4" />
                        Generate Service
                    </Button>
                )}
            </div>
            <div className="rounded-md border min-h-[300px]">
                {capturedRequests.length > 0 ? (
                    <Table>
                        <TableHeader>
                            <TableRow>
                                <TableHead>Method</TableHead>
                                <TableHead>Path</TableHead>
                                <TableHead className="text-right">Status</TableHead>
                            </TableRow>
                        </TableHeader>
                        <TableBody>
                            {capturedRequests.map((req) => (
                                <TableRow key={req.id}>
                                    <TableCell>
                                        <Badge variant="secondary" className="font-mono">{req.method}</Badge>
                                    </TableCell>
                                    <TableCell className="font-mono text-sm">{req.path}</TableCell>
                                    <TableCell className="text-right">
                                        <Badge variant="outline" className={getStatusColor(req.status)}>{req.status}</Badge>
                                    </TableCell>
                                </TableRow>
                            ))}
                        </TableBody>
                    </Table>
                ) : (
                    <div className="flex items-center justify-center h-full min-h-[300px]">
                        <div className="text-center text-muted-foreground">
                            <RadioTower className="mx-auto h-12 w-12" />
                            <p className="mt-2">{isRecording ? 'Waiting for requests...' : 'Start recording to capture traffic.'}</p>
                        </div>
                    </div>
                )}
            </div>
        </div>
      </CardContent>
    </Card>
  );
}
