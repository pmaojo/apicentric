'use client';

import * as React from 'react';
import { RouteConfig } from './layout';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { Textarea } from '@/components/ui/textarea';
import dynamic from 'next/dynamic';
import { Loader2 } from 'lucide-react';

const Editor = dynamic(() => import('@monaco-editor/react'), {
  loading: () => <Loader2 className="h-4 w-4 animate-spin" />,
  ssr: false,
});

interface RouteEditorProps {
  route: RouteConfig;
  onChange: (route: RouteConfig) => void;
}

export function RouteEditor({ route, onChange }: RouteEditorProps) {
  
  const handleMethodChange = (method: string) => {
    onChange({ ...route, method });
  };

  const handlePathChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    onChange({ ...route, path: e.target.value });
  };

  const handleStatusChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    onChange({
      ...route,
      response: { ...route.response, status: parseInt(e.target.value) || 200 }
    });
  };

  const handleBodyChange = (value: string | undefined) => {
    onChange({
      ...route,
      response: { ...route.response, body: value || '' }
    });
  };

  return (
    <div className="flex flex-col h-full bg-background">
      {/* Route Header Settings */}
      <div className="p-6 border-b bg-card space-y-4">
        <div className="flex gap-4">
           <div className="w-32">
             <Label className="text-xs text-muted-foreground mb-1.5 block">Method</Label>
             <Select value={route.method} onValueChange={handleMethodChange}>
                <SelectTrigger>
                    <SelectValue />
                </SelectTrigger>
                <SelectContent>
                    <SelectItem value="GET">GET</SelectItem>
                    <SelectItem value="POST">POST</SelectItem>
                    <SelectItem value="PUT">PUT</SelectItem>
                    <SelectItem value="DELETE">DELETE</SelectItem>
                    <SelectItem value="PATCH">PATCH</SelectItem>
                    <SelectItem value="OPTIONS">OPTIONS</SelectItem>
                </SelectContent>
             </Select>
           </div>
           <div className="flex-1">
             <Label className="text-xs text-muted-foreground mb-1.5 block">Path</Label>
             <Input value={route.path} onChange={handlePathChange} placeholder="/api/resource" className="font-mono" />
           </div>
        </div>
      </div>

      {/* Tabs for Response config */}
      <Tabs defaultValue="body" className="flex-1 flex flex-col overflow-hidden">
        <div className="border-b px-6 bg-muted/10">
            <TabsList className="bg-transparent h-12 w-full justify-start gap-4 rounded-none p-0">
            <TabsTrigger 
                value="body" 
                className="data-[state=active]:bg-transparent data-[state=active]:border-b-2 data-[state=active]:border-primary data-[state=active]:shadow-none rounded-none px-2 h-full"
            >
                Status & Body
            </TabsTrigger>
            <TabsTrigger 
                value="headers"
                className="data-[state=active]:bg-transparent data-[state=active]:border-b-2 data-[state=active]:border-primary data-[state=active]:shadow-none rounded-none px-2 h-full"
            >
                Headers
            </TabsTrigger>
            <TabsTrigger 
                value="settings"
                className="data-[state=active]:bg-transparent data-[state=active]:border-b-2 data-[state=active]:border-primary data-[state=active]:shadow-none rounded-none px-2 h-full"
            >
                Settings
            </TabsTrigger>
            </TabsList>
        </div>

        <TabsContent value="body" className="flex-1 p-6 m-0 overflow-y-auto space-y-6">
            <div className="space-y-4">
                <div className="w-48">
                    <Label className="mb-2 block">Status Code</Label>
                    <div className="flex items-center gap-2">
                        <Input 
                            type="number" 
                            value={route.response.status} 
                            onChange={handleStatusChange}
                            className="font-mono"
                        />
                        <div className="text-sm text-muted-foreground whitespace-nowrap">
                            {getStatusText(route.response.status)}
                        </div>
                    </div>
                </div>

                <div className="h-[400px] border rounded-md overflow-hidden bg-muted/20">
                    <Editor
                        height="100%"
                        defaultLanguage="json"
                        value={route.response.body}
                        onChange={handleBodyChange}
                        theme="vs-dark"
                        options={{
                            minimap: { enabled: false },
                            lineNumbers: 'on',
                            scrollBeyondLastLine: false,
                            automaticLayout: true,
                            tabSize: 2,
                        }}
                    />
                </div>
            </div>
        </TabsContent>

        <TabsContent value="headers" className="flex-1 p-6 m-0">
            <div className="flex items-center justify-center h-full text-muted-foreground">
                Headers editor coming soon...
            </div>
        </TabsContent>
      </Tabs>
    </div>
  );
}

function getStatusText(code: number): string {
    const statusMap: Record<number, string> = {
        200: 'OK',
        201: 'Created',
        204: 'No Content',
        400: 'Bad Request',
        401: 'Unauthorized',
        403: 'Forbidden',
        404: 'Not Found',
        500: 'Internal Server Error'
    };
    return statusMap[code] || 'Unknown Status';
}
