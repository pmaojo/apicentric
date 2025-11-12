'use client';

import * as React from 'react';
import { Code, Download, Type, Clipboard, Check, AlertTriangle } from 'lucide-react';
import Editor from '@monaco-editor/react';
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from '@/components/ui/card';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
import { Button } from '@/components/ui/button';
import { Separator } from '@/components/ui/separator';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import type { Service } from '@/lib/types';
import { Skeleton } from '../ui/skeleton';
import { useToast } from '@/hooks/use-toast';
import { useMutation } from '@tanstack/react-query';
import { generateTypeScript, generateReactQuery, generateAxios } from '@/services/api';

type CodeGeneratorProps = {
    services: Service[];
    isLoading: boolean;
}

type CodeTarget = 'typescript' | 'react-query' | 'axios';

export function CodeGenerator({ services, isLoading: isLoadingServices }: CodeGeneratorProps) {
  const [selectedService, setSelectedService] = React.useState<Service | null>(null);
  const [selectedTarget, setSelectedTarget] = React.useState<CodeTarget>('typescript');
  const [isCopied, setIsCopied] = React.useState(false);
  const { toast } = useToast();

  // TypeScript generation
  const { mutate: generateTS, data: tsCode, isPending: isGeneratingTS, error: tsError } = useMutation<string, Error, string>({
    mutationFn: (serviceName: string) => generateTypeScript(serviceName),
    onSuccess: () => {
        toast({
            title: 'TypeScript Types Generated',
            description: 'TypeScript interfaces and types have been successfully generated.',
        });
    },
    onError: (err) => {
        toast({
            variant: "destructive",
            title: "Generation Failed",
            description: err.message,
        });
    },
  });

  // React Query generation
  const { mutate: generateRQ, data: rqCode, isPending: isGeneratingRQ, error: rqError } = useMutation<string, Error, string>({
    mutationFn: (serviceName: string) => generateReactQuery(serviceName),
    onSuccess: () => {
        toast({
            title: 'React Query Hooks Generated',
            description: 'React Query hooks have been successfully generated.',
        });
    },
    onError: (err) => {
        toast({
            variant: "destructive",
            title: "Generation Failed",
            description: err.message,
        });
    },
  });

  // Axios generation
  const { mutate: generateAx, data: axCode, isPending: isGeneratingAx, error: axError } = useMutation<string, Error, string>({
    mutationFn: (serviceName: string) => generateAxios(serviceName),
    onSuccess: () => {
        toast({
            title: 'Axios Client Generated',
            description: 'Axios client code has been successfully generated.',
        });
    },
    onError: (err) => {
        toast({
            variant: "destructive",
            title: "Generation Failed",
            description: err.message,
        });
    },
  });

  const handleGenerate = (target: CodeTarget) => {
    if (!selectedService) {
        toast({
            variant: 'destructive',
            title: 'No Service Selected',
            description: 'Please select a service before generating code.',
        });
        return;
    }

    switch (target) {
      case 'typescript':
        generateTS(selectedService.name);
        break;
      case 'react-query':
        generateRQ(selectedService.name);
        break;
      case 'axios':
        generateAx(selectedService.name);
        break;
    }
  };

  const getCurrentCode = () => {
    switch (selectedTarget) {
      case 'typescript':
        return tsCode;
      case 'react-query':
        return rqCode;
      case 'axios':
        return axCode;
      default:
        return undefined;
    }
  };

  const getCurrentError = () => {
    switch (selectedTarget) {
      case 'typescript':
        return tsError;
      case 'react-query':
        return rqError;
      case 'axios':
        return axError;
      default:
        return undefined;
    }
  };

  const isGenerating = isGeneratingTS || isGeneratingRQ || isGeneratingAx;
  const generatedCode = getCurrentCode();
  const error = getCurrentError();

  const handleCopy = () => {
    if (generatedCode) {
      navigator.clipboard.writeText(generatedCode);
      setIsCopied(true);
      toast({ title: 'Copied to clipboard!' });
      setTimeout(() => setIsCopied(false), 2000);
    }
  };

  const handleDownload = () => {
    if (!generatedCode || !selectedService) return;

    const fileExtension = selectedTarget === 'typescript' ? 'ts' : 'tsx';
    const fileName = `${selectedService.name}-${selectedTarget}.${fileExtension}`;
    
    const blob = new Blob([generatedCode], { type: 'text/plain' });
    const url = URL.createObjectURL(blob);
    const link = document.createElement('a');
    link.href = url;
    link.download = fileName;
    document.body.appendChild(link);
    link.click();
    document.body.removeChild(link);
    URL.revokeObjectURL(url);
    
    toast({ 
      title: 'Download Started',
      description: `Downloading ${fileName}...`
    });
  };

  const handleDownloadAll = async () => {
    if (!selectedService) {
      toast({
        variant: 'destructive',
        title: 'No Service Selected',
        description: 'Please select a service before downloading.',
      });
      return;
    }

    try {
      // Generate all code types
      const [types, hooks, client] = await Promise.all([
        generateTypeScript(selectedService.name).catch(() => null),
        generateReactQuery(selectedService.name).catch(() => null),
        generateAxios(selectedService.name).catch(() => null),
      ]);

      const files: Array<{ name: string; content: string }> = [];
      
      if (types) {
        files.push({ name: `${selectedService.name}-types.ts`, content: types });
      }
      if (hooks) {
        files.push({ name: `${selectedService.name}-hooks.tsx`, content: hooks });
      }
      if (client) {
        files.push({ name: `${selectedService.name}-client.ts`, content: client });
      }

      if (files.length === 0) {
        toast({
          variant: 'destructive',
          title: 'Generation Failed',
          description: 'Failed to generate any code files.',
        });
        return;
      }

      // Download each file
      files.forEach(file => {
        const blob = new Blob([file.content], { type: 'text/plain' });
        const url = URL.createObjectURL(blob);
        const link = document.createElement('a');
        link.href = url;
        link.download = file.name;
        document.body.appendChild(link);
        link.click();
        document.body.removeChild(link);
        URL.revokeObjectURL(url);
      });

      toast({
        title: 'Download Complete',
        description: `Downloaded ${files.length} file(s) successfully.`,
      });
    } catch (err) {
      toast({
        variant: 'destructive',
        title: 'Download Failed',
        description: err instanceof Error ? err.message : 'An error occurred',
      });
    }
  };

  return (
    <div className="grid grid-cols-1 gap-8 lg:grid-cols-2">
      <Card>
        <CardHeader>
          <CardTitle>Client Code Generator</CardTitle>
          <CardDescription>
            Generate TypeScript types, React Query hooks, and Axios client from your service definitions.
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-6">
          <div className="space-y-2">
            <label className="text-sm font-medium">Select Service</label>
            {isLoadingServices ? (
              <Skeleton className="h-10 w-full" />
            ) : (
              <Select 
                onValueChange={(value) => setSelectedService(services.find(s => s.name === value) || null)} 
                value={selectedService?.name || ""}
              >
                <SelectTrigger>
                  <SelectValue placeholder="Select a service..." />
                </SelectTrigger>
                <SelectContent>
                  {services.map((service) => (
                    <SelectItem key={service.id} value={service.name}>
                      {service.name} {service.version && `(v${service.version})`}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            )}
          </div>

          <Separator />

          <div className="space-y-2">
            <label className="text-sm font-medium">Select Target</label>
            <Tabs value={selectedTarget} onValueChange={(value) => setSelectedTarget(value as CodeTarget)}>
              <TabsList className="grid w-full grid-cols-3">
                <TabsTrigger value="typescript">
                  <Type className="h-4 w-4 mr-2" />
                  TypeScript
                </TabsTrigger>
                <TabsTrigger value="react-query">
                  <Code className="h-4 w-4 mr-2" />
                  React Query
                </TabsTrigger>
                <TabsTrigger value="axios">
                  <Code className="h-4 w-4 mr-2" />
                  Axios
                </TabsTrigger>
              </TabsList>
              
              <TabsContent value="typescript" className="space-y-4">
                <p className="text-sm text-muted-foreground">
                  Generate TypeScript interfaces and types for your API request bodies and responses.
                </p>
                <Button 
                  className="w-full" 
                  onClick={() => handleGenerate('typescript')} 
                  disabled={isGenerating || !selectedService}
                >
                  {isGeneratingTS ? 'Generating...' : 'Generate TypeScript Types'}
                </Button>
              </TabsContent>
              
              <TabsContent value="react-query" className="space-y-4">
                <p className="text-sm text-muted-foreground">
                  Generate React Query hooks for fetching, creating, and updating data with proper TypeScript types.
                </p>
                <Button 
                  className="w-full" 
                  onClick={() => handleGenerate('react-query')} 
                  disabled={isGenerating || !selectedService}
                >
                  {isGeneratingRQ ? 'Generating...' : 'Generate React Query Hooks'}
                </Button>
              </TabsContent>
              
              <TabsContent value="axios" className="space-y-4">
                <p className="text-sm text-muted-foreground">
                  Generate an Axios client with all API endpoints and TypeScript types.
                </p>
                <Button 
                  className="w-full" 
                  onClick={() => handleGenerate('axios')} 
                  disabled={isGenerating || !selectedService}
                >
                  {isGeneratingAx ? 'Generating...' : 'Generate Axios Client'}
                </Button>
              </TabsContent>
            </Tabs>
          </div>
        </CardContent>
      </Card>
      
      <Card className="flex flex-col">
        <CardHeader>
          <div className="flex items-center justify-between">
            <div>
              <CardTitle>Generated Code</CardTitle>
              <CardDescription>
                {generatedCode 
                  ? `${selectedTarget === 'typescript' ? 'TypeScript Types' : selectedTarget === 'react-query' ? 'React Query Hooks' : 'Axios Client'} for ${selectedService?.name}`
                  : 'The generated client code will appear here.'
                }
              </CardDescription>
            </div>
            <div className="flex gap-2">
              {generatedCode && (
                <>
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={handleCopy}
                    disabled={!generatedCode}
                  >
                    {isCopied ? (
                      <>
                        <Check className="h-4 w-4 mr-2" />
                        Copied
                      </>
                    ) : (
                      <>
                        <Clipboard className="h-4 w-4 mr-2" />
                        Copy
                      </>
                    )}
                  </Button>
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={handleDownload}
                    disabled={!generatedCode}
                  >
                    <Download className="h-4 w-4 mr-2" />
                    Download
                  </Button>
                </>
              )}
              <Button
                variant="outline"
                size="sm"
                onClick={handleDownloadAll}
                disabled={!selectedService || isGenerating}
              >
                <Download className="h-4 w-4 mr-2" />
                Download All
              </Button>
            </div>
          </div>
        </CardHeader>
        <CardContent className="flex-grow">
          <div className="relative h-full min-h-[400px] rounded-md border bg-muted/30">
            {isGenerating && (
              <div className="space-y-2 p-4">
                <Skeleton className="h-4 w-3/4" />
                <Skeleton className="h-4 w-1/2" />
                <Skeleton className="h-4 w-5/6" />
                <Skeleton className="h-4 w-2/3" />
                <Skeleton className="h-4 w-4/5" />
                <Skeleton className="h-4 w-3/5" />
              </div>
            )}
            {error && !isGenerating && (
                <div className="flex flex-col items-center justify-center h-full text-center text-destructive p-4">
                    <AlertTriangle className="h-10 w-10 mb-4" />
                    <p className="font-semibold">Generation Failed</p>
                    <p className="text-sm">{error.message}</p>
                </div>
            )}
            {!isGenerating && !generatedCode && !error && (
              <div className="flex flex-col items-center justify-center h-full text-center text-muted-foreground p-4">
                <Code className="h-12 w-12 mb-4 opacity-50" />
                <p className="font-medium">No Code Generated Yet</p>
                <p className="text-sm mt-2">Select a service and target, then click generate.</p>
              </div>
            )}
            {generatedCode && !isGenerating && (
              <Editor
                height="400px"
                language="typescript"
                value={generatedCode}
                theme="vs-dark"
                options={{
                  readOnly: true,
                  minimap: { enabled: false },
                  scrollBeyondLastLine: false,
                  fontSize: 13,
                  lineNumbers: 'on',
                  renderLineHighlight: 'none',
                  scrollbar: {
                    vertical: 'auto',
                    horizontal: 'auto',
                  },
                }}
              />
            )}
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
