'use client';

import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { RadioTower, PlayCircle, Square, FilePlus, Copy, Check } from 'lucide-react';
import { Input } from '@/components/ui/input';
import * as React from 'react';
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '../ui/table';
import { Badge } from '../ui/badge';
import { useToast } from '@/hooks/use-toast';
import { useWebSocketSubscription, type RecordingCapture } from '@/providers/websocket-provider';
import * as api from '@/services/api';
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
  DialogFooter,
} from '@/components/ui/dialog';
import { ScrollArea } from '@/components/ui/scroll-area';
import { Separator } from '@/components/ui/separator';
import { Textarea } from '@/components/ui/textarea';
import { Label } from '@/components/ui/label';

type CapturedRequest = {
  id: string;
  method: string;
  path: string;
  status: number;
  headers?: Record<string, string>;
  body?: string;
  response_status?: number;
  response_headers?: Record<string, string>;
  response_body?: string;
};

const WS_URL = process.env.NEXT_PUBLIC_WS_URL || 'ws://localhost:8080/ws';

export function Recording() {
  const [isRecording, setIsRecording] = React.useState(false);
  const [targetUrl, setTargetUrl] = React.useState('https://api.example.com');
  const [proxyUrl, setProxyUrl] = React.useState<string | null>(null);
  const [proxyPort, setProxyPort] = React.useState<number | null>(null);
  const [sessionId, setSessionId] = React.useState<string | null>(null);
  const [capturedRequests, setCapturedRequests] = React.useState<CapturedRequest[]>([]);
  const [filteredRequests, setFilteredRequests] = React.useState<CapturedRequest[]>([]);
  const [selectedRequest, setSelectedRequest] = React.useState<CapturedRequest | null>(null);
  const [copiedProxy, setCopiedProxy] = React.useState(false);
  const [isLoading, setIsLoading] = React.useState(false);
  const [showGenerateDialog, setShowGenerateDialog] = React.useState(false);
  const [generatedYaml, setGeneratedYaml] = React.useState<string>('');
  const [serviceName, setServiceName] = React.useState<string>('');
  const [isGenerating, setIsGenerating] = React.useState(false);
  const [isSaving, setIsSaving] = React.useState(false);
  const { toast } = useToast();

  // Subscribe to recording captures via WebSocket  
  useWebSocketSubscription('recording_capture', (captured: RecordingCapture) => {
    setCapturedRequests(prev => [...prev, {
      id: `req-${Date.now()}-${Math.random()}`,
      method: captured.method,
      path: captured.path,
      status: captured.response_status,
      headers: captured.headers,
      body: captured.body,
      response_status: captured.response_status,
      response_headers: captured.response_headers,
      response_body: captured.response_body,
    }]);
  }, []);

  const isConnected = true; // Now managed by WebSocketProvider

  // Check recording status on mount
  React.useEffect(() => {
    const checkStatus = async () => {
      try {
        const status = await api.getRecordingStatus();
        if (status.is_active) {
          setIsRecording(true);
          setSessionId(status.session_id || null);
          setProxyUrl(status.proxy_url || null);
          setProxyPort(status.proxy_port || null);
        }
      } catch (error) {
        // Recording not active or error checking status
        console.error('Failed to check recording status:', error);
      }
    };
    checkStatus();
  }, []);

  const handleStartRecording = async () => {
    if (!targetUrl) {
      toast({
        title: "Error",
        description: "Please enter a target URL",
        variant: "destructive",
      });
      return;
    }

    setIsLoading(true);
    try {
      const response = await api.startRecording(targetUrl);
      setIsRecording(true);
      setSessionId(response.session_id);
      setProxyUrl(response.proxy_url);
      setProxyPort(response.proxy_port);
      setCapturedRequests([]);
      
      toast({
        title: "Recording Started",
        description: `Proxy listening on ${response.proxy_url}`,
      });
    } catch (error) {
      toast({
        title: "Failed to Start Recording",
        description: error instanceof Error ? error.message : "Unknown error",
        variant: "destructive",
      });
    } finally {
      setIsLoading(false);
    }
  };

  const handleStopRecording = async () => {
    setIsLoading(true);
    try {
      const response = await api.stopRecording();
      setIsRecording(false);
      setSessionId(null);
      setProxyUrl(null);
      setProxyPort(null);
      
      // Update captured requests with final data
      if (response.captured_requests && response.captured_requests.length > 0) {
        setCapturedRequests(response.captured_requests.map((req, idx) => ({
          id: `req-${Date.now()}-${idx}`,
          method: req.method,
          path: req.path,
          status: req.response_status,
          headers: req.headers,
          body: req.body,
          response_status: req.response_status,
          response_headers: req.response_headers,
          response_body: req.response_body,
        })));
      }
      
      toast({
        title: "Recording Stopped",
        description: `Captured ${response.captured_requests?.length || 0} requests`,
      });
    } catch (error) {
      toast({
        title: "Failed to Stop Recording",
        description: error instanceof Error ? error.message : "Unknown error",
        variant: "destructive",
      });
    } finally {
      setIsLoading(false);
    }
  };

  const handleToggleRecording = () => {
    if (isRecording) {
      handleStopRecording();
    } else {
      handleStartRecording();
    }
  };

  const handleCopyProxyUrl = async () => {
    if (proxyUrl) {
      try {
        await navigator.clipboard.writeText(proxyUrl);
        setCopiedProxy(true);
        setTimeout(() => setCopiedProxy(false), 2000);
        toast({
          title: "Copied",
          description: "Proxy URL copied to clipboard",
        });
      } catch (error) {
        toast({
          title: "Failed to Copy",
          description: "Could not copy to clipboard",
          variant: "destructive",
        });
      }
    }
  };

  const handleGenerateService = async () => {
    if (capturedRequests.length === 0) {
      toast({
        title: "No Requests",
        description: "No captured requests to generate service from",
        variant: "destructive",
      });
      return;
    }

    // Generate a default service name from target URL
    const defaultName = targetUrl
      .replace(/^https?:\/\//, '')
      .replace(/[^a-zA-Z0-9]/g, '-')
      .toLowerCase()
      .substring(0, 30);
    
    setServiceName(defaultName);
    setShowGenerateDialog(true);
    setIsGenerating(true);

    try {
      const response = await api.generateServiceFromRecording(defaultName);
      
      // Get the service YAML
      const serviceData = await api.getService(response.name);
      
      // Create YAML representation (simplified - in real implementation would use proper YAML serialization)
      const yaml = `name: ${serviceData.name}
version: 1.0.0
description: Generated from recorded API traffic
server:
  port: ${serviceData.port}
  base_path: /

endpoints:
${serviceData.endpoints.map(ep => `  - method: ${ep.method}
    path: ${ep.path}
    responses:
      - status: 200
        body: {}`).join('\n')}
`;
      
      setGeneratedYaml(yaml);
      
      toast({
        title: "Service Generated",
        description: `Generated service definition with ${serviceData.endpoints.length} endpoints`,
      });
    } catch (error) {
      toast({
        title: "Generation Failed",
        description: error instanceof Error ? error.message : "Failed to generate service",
        variant: "destructive",
      });
      setShowGenerateDialog(false);
    } finally {
      setIsGenerating(false);
    }
  };

  const handleSaveGeneratedService = async () => {
    if (!serviceName || !generatedYaml) {
      toast({
        title: "Invalid Input",
        description: "Service name and YAML are required",
        variant: "destructive",
      });
      return;
    }

    setIsSaving(true);
    try {
      await api.createService(generatedYaml, `${serviceName}.yaml`);
      
      toast({
        title: "Service Saved",
        description: `Service "${serviceName}" has been created successfully`,
      });
      
      setShowGenerateDialog(false);
      setGeneratedYaml('');
      setServiceName('');
    } catch (error) {
      toast({
        title: "Save Failed",
        description: error instanceof Error ? error.message : "Failed to save service",
        variant: "destructive",
      });
    } finally {
      setIsSaving(false);
    }
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
                    data-testid="recording-target-url-input"
                    placeholder="https://api.example.com" 
                    disabled={isRecording || isLoading}
                    value={targetUrl}
                    onChange={(e) => setTargetUrl(e.target.value)}
                />
                <Button 
                  data-testid={isRecording ? "stop-recording-button" : "start-recording-button"}
                  onClick={handleToggleRecording} 
                  disabled={!targetUrl || isLoading}
                >
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
            
            {/* Recording Status Indicator */}
            {isRecording && (
              <div className="space-y-2" data-testid="recording-status">
                <div className="flex items-center gap-2 text-sm">
                  <RadioTower className="h-4 w-4 animate-pulse text-primary" />
                  <span className="text-muted-foreground">
                    Recording active {isConnected ? '(connected)' : '(connecting...)'}
                  </span>
                  {sessionId && (
                    <Badge variant="outline" className="ml-2 font-mono text-xs">
                      {sessionId.substring(0, 8)}
                    </Badge>
                  )}
                </div>
                
                {/* Proxy URL Display with Copy Button */}
                {proxyUrl && (
                  <div className="flex items-center gap-2 p-3 bg-muted rounded-md">
                    <div className="flex-1">
                      <p className="text-xs text-muted-foreground mb-1">Proxy URL</p>
                      <p className="font-mono text-sm" data-testid="proxy-url">{proxyUrl}</p>
                      {proxyPort && (
                        <p className="text-xs text-muted-foreground mt-1">
                          Port: {proxyPort}
                        </p>
                      )}
                    </div>
                    <Button
                      variant="outline"
                      size="sm"
                      onClick={handleCopyProxyUrl}
                      className="shrink-0"
                    >
                      {copiedProxy ? (
                        <>
                          <Check className="h-4 w-4 mr-1" />
                          Copied
                        </>
                      ) : (
                        <>
                          <Copy className="h-4 w-4 mr-1" />
                          Copy
                        </>
                      )}
                    </Button>
                  </div>
                )}
              </div>
            )}
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
            
            {/* Filtering Controls */}
            {capturedRequests.length > 0 && (
              <CapturedRequestsFilters
                requests={capturedRequests}
                onFilterChange={setFilteredRequests}
              />
            )}
            
            {/* Captured Requests Table */}
            <CapturedRequestsTable
              requests={filteredRequests.length > 0 ? filteredRequests : capturedRequests}
              isRecording={isRecording}
              onViewDetails={setSelectedRequest}
            />
        </div>
      </CardContent>
      
      {/* Request Detail Dialog */}
      <RequestDetailDialog
        request={selectedRequest}
        open={!!selectedRequest}
        onClose={() => setSelectedRequest(null)}
      />
      
      {/* Generate Service Dialog */}
      <Dialog open={showGenerateDialog} onOpenChange={setShowGenerateDialog}>
        <DialogContent className="max-w-4xl max-h-[80vh]">
          <DialogHeader>
            <DialogTitle>Generate Service from Recording</DialogTitle>
            <DialogDescription>
              Review and edit the generated service definition before saving
            </DialogDescription>
          </DialogHeader>
          
          <div className="space-y-4">
            <div className="space-y-2">
              <Label htmlFor="service-name">Service Name</Label>
              <Input
                id="service-name"
                value={serviceName}
                onChange={(e) => setServiceName(e.target.value)}
                placeholder="my-api-service"
                disabled={isGenerating || isSaving}
              />
            </div>
            
            <div className="space-y-2">
              <Label htmlFor="service-yaml">Service Definition (YAML)</Label>
              {isGenerating ? (
                <div className="flex items-center justify-center h-[400px] bg-muted rounded-md">
                  <div className="text-center">
                    <RadioTower className="mx-auto h-8 w-8 animate-pulse text-primary mb-2" />
                    <p className="text-sm text-muted-foreground">Generating service definition...</p>
                  </div>
                </div>
              ) : (
                <Textarea
                  id="service-yaml"
                  value={generatedYaml}
                  onChange={(e) => setGeneratedYaml(e.target.value)}
                  className="font-mono text-sm h-[400px]"
                  placeholder="Service YAML will appear here..."
                  disabled={isSaving}
                />
              )}
            </div>
          </div>
          
          <DialogFooter>
            <Button
              variant="outline"
              onClick={() => setShowGenerateDialog(false)}
              disabled={isGenerating || isSaving}
            >
              Cancel
            </Button>
            <Button
              onClick={handleSaveGeneratedService}
              disabled={isGenerating || isSaving || !generatedYaml || !serviceName}
            >
              {isSaving ? 'Saving...' : 'Save Service'}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </Card>
  );
}

// Filtering component for captured requests
function CapturedRequestsFilters({
  requests,
  onFilterChange,
}: {
  requests: CapturedRequest[];
  onFilterChange: (filtered: CapturedRequest[]) => void;
}) {
  const [methodFilter, setMethodFilter] = React.useState<string>('all');
  const [statusFilter, setStatusFilter] = React.useState<string>('all');
  const [pathSearch, setPathSearch] = React.useState<string>('');

  React.useEffect(() => {
    let filtered = requests;

    // Filter by method
    if (methodFilter !== 'all') {
      filtered = filtered.filter(req => req.method === methodFilter);
    }

    // Filter by status
    if (statusFilter !== 'all') {
      if (statusFilter === '2xx') {
        filtered = filtered.filter(req => req.status >= 200 && req.status < 300);
      } else if (statusFilter === '4xx') {
        filtered = filtered.filter(req => req.status >= 400 && req.status < 500);
      } else if (statusFilter === '5xx') {
        filtered = filtered.filter(req => req.status >= 500);
      }
    }

    // Filter by path search
    if (pathSearch) {
      filtered = filtered.filter(req => 
        req.path.toLowerCase().includes(pathSearch.toLowerCase())
      );
    }

    onFilterChange(filtered);
  }, [methodFilter, statusFilter, pathSearch, requests, onFilterChange]);

  const methods = Array.from(new Set(requests.map(r => r.method)));

  return (
    <div className="flex gap-2 flex-wrap">
      <Input
        placeholder="Search path..."
        value={pathSearch}
        onChange={(e) => setPathSearch(e.target.value)}
        className="max-w-xs"
      />
      <select
        value={methodFilter}
        onChange={(e) => setMethodFilter(e.target.value)}
        className="px-3 py-2 border rounded-md text-sm"
        aria-label="Filter by HTTP method"
      >
        <option value="all">All Methods</option>
        {methods.map(method => (
          <option key={method} value={method}>{method}</option>
        ))}
      </select>
      <select
        value={statusFilter}
        onChange={(e) => setStatusFilter(e.target.value)}
        className="px-3 py-2 border rounded-md text-sm"
        aria-label="Filter by status code"
      >
        <option value="all">All Status</option>
        <option value="2xx">2xx Success</option>
        <option value="4xx">4xx Client Error</option>
        <option value="5xx">5xx Server Error</option>
      </select>
    </div>
  );
}

// Table component for displaying captured requests
function CapturedRequestsTable({
  requests,
  isRecording,
  onViewDetails,
}: {
  requests: CapturedRequest[];
  isRecording: boolean;
  onViewDetails: (request: CapturedRequest) => void;
}) {
  const getStatusColor = (status: number) => {
    if (status >= 500) return 'bg-red-500/20 text-red-400 border-red-500/30';
    if (status >= 400) return 'bg-yellow-500/20 text-yellow-400 border-yellow-500/30';
    if (status >= 200) return 'bg-green-500/20 text-green-400 border-green-500/30';
    return 'bg-gray-500/20 text-gray-400 border-gray-500/30';
  };

  return (
    <div className="rounded-md border min-h-[300px]">
      {requests.length > 0 ? (
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>Method</TableHead>
              <TableHead>Path</TableHead>
              <TableHead className="text-right">Status</TableHead>
              <TableHead className="text-right">Actions</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {requests.map((req) => (
              <TableRow key={req.id} className="cursor-pointer hover:bg-muted/50">
                <TableCell>
                  <Badge variant="secondary" className="font-mono">{req.method}</Badge>
                </TableCell>
                <TableCell className="font-mono text-sm">{req.path}</TableCell>
                <TableCell className="text-right">
                  <Badge variant="outline" className={getStatusColor(req.status)}>{req.status}</Badge>
                </TableCell>
                <TableCell className="text-right">
                  <Button
                    variant="ghost"
                    size="sm"
                    onClick={() => onViewDetails(req)}
                  >
                    View Details
                  </Button>
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
  );
}

// Dialog component for viewing request details
function RequestDetailDialog({
  request,
  open,
  onClose,
}: {
  request: CapturedRequest | null;
  open: boolean;
  onClose: () => void;
}) {
  const getStatusColor = (status: number) => {
    if (status >= 500) return 'bg-red-500/20 text-red-400 border-red-500/30';
    if (status >= 400) return 'bg-yellow-500/20 text-yellow-400 border-yellow-500/30';
    if (status >= 200) return 'bg-green-500/20 text-green-400 border-green-500/30';
    return 'bg-gray-500/20 text-gray-400 border-gray-500/30';
  };

  return (
    <Dialog open={open} onOpenChange={onClose}>
      <DialogContent className="max-w-4xl max-h-[80vh]">
        <DialogHeader>
          <DialogTitle>Request Details</DialogTitle>
          <DialogDescription>
            {request && (
              <div className="flex items-center gap-2 mt-2">
                <Badge variant="outline" className="font-mono">
                  {request.method}
                </Badge>
                <span className="font-mono text-sm">{request.path}</span>
                <Badge
                  variant="outline"
                  className={getStatusColor(request.status)}
                >
                  {request.status}
                </Badge>
              </div>
            )}
          </DialogDescription>
        </DialogHeader>
        {request && (
          <ScrollArea className="h-[60vh]">
            <div className="space-y-4">
              {/* Request Headers */}
              {request.headers && Object.keys(request.headers).length > 0 && (
                <>
                  <div>
                    <h3 className="font-semibold mb-2">Request Headers</h3>
                    <pre className="bg-muted p-3 rounded-md text-xs overflow-x-auto">
                      {JSON.stringify(request.headers, null, 2)}
                    </pre>
                  </div>
                  <Separator />
                </>
              )}

              {/* Request Body */}
              {request.body && (
                <>
                  <div>
                    <h3 className="font-semibold mb-2">Request Body</h3>
                    <pre className="bg-muted p-3 rounded-md text-xs overflow-x-auto">
                      {request.body}
                    </pre>
                  </div>
                  <Separator />
                </>
              )}

              {/* Response Headers */}
              {request.response_headers && Object.keys(request.response_headers).length > 0 && (
                <>
                  <div>
                    <h3 className="font-semibold mb-2">Response Headers</h3>
                    <pre className="bg-muted p-3 rounded-md text-xs overflow-x-auto">
                      {JSON.stringify(request.response_headers, null, 2)}
                    </pre>
                  </div>
                  <Separator />
                </>
              )}

              {/* Response Body */}
              {request.response_body && (
                <div>
                  <h3 className="font-semibold mb-2">Response Body</h3>
                  <pre className="bg-muted p-3 rounded-md text-xs overflow-x-auto">
                    {request.response_body}
                  </pre>
                </div>
              )}
            </div>
          </ScrollArea>
        )}
      </DialogContent>
    </Dialog>
  );
}
