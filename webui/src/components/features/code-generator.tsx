'use client';

import * as React from 'react';
import { Code, ToyBrick, Type, Clipboard, Check, AlertTriangle } from 'lucide-react';
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
import type { ApiService, Service } from '@/lib/types';
import { Skeleton } from '../ui/skeleton';
import { useToast } from '@/hooks/use-toast';
import { useMutation } from '@tanstack/react-query';
import { generateCode } from '@/services/api';

type CodeGeneratorProps = {
    services: Service[];
    isLoading: boolean;
}

export function CodeGenerator({ services, isLoading: isLoadingServices }: CodeGeneratorProps) {
  const [selectedService, setSelectedService] = React.useState<Service | null>(null);
  const [isCopied, setIsCopied] = React.useState(false);
  const { toast } = useToast();

  const { mutate: generate, data: generatedCode, isPending: isGenerating, error } = useMutation<string, Error, { definition: string; target: 'typescript' | 'react-query' | 'react-components' }>({
    mutationFn: ({ definition, target }) => generateCode(definition, target),
    onSuccess: () => {
        toast({
            title: 'Code Generation Complete',
            description: 'The client code has been successfully generated.',
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

  const handleGenerate = (target: 'typescript' | 'react-query' | 'react-components') => {
    if (!selectedService) {
        toast({
            variant: 'destructive',
            title: 'No Service Selected',
            description: 'Please select a service before generating code.',
        });
        return;
    }
    generate({ definition: selectedService.definition, target });
  };

  const handleCopy = () => {
    if (generatedCode) {
      navigator.clipboard.writeText(generatedCode);
      setIsCopied(true);
      toast({ title: 'Copied to clipboard!' });
      setTimeout(() => setIsCopied(false), 2000);
    }
  };

  return (
    <div className="grid grid-cols-1 gap-8 lg:grid-cols-2">
      <Card>
        <CardHeader>
          <CardTitle>Client Code Generator</CardTitle>
          <CardDescription>
            Generate TypeScript types, React Query hooks, and React components from your service definitions.
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-6">
          <div className="space-y-2">
            <label className="text-sm font-medium">Select Service Definition</label>
            {isLoadingServices ? (
              <Skeleton className="h-10 w-full" />
            ) : (
              <Select onValueChange={(value) => setSelectedService(services.find(s => s.name === value) || null)} value={selectedService?.name || ""}>
                <SelectTrigger>
                  <SelectValue placeholder="Select a service..." />
                </SelectTrigger>
                <SelectContent>
                  {services.map((service) => (
                    <SelectItem key={service.id} value={service.name}>
                      {service.name} (v{service.version})
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            )}
          </div>

          <Separator />

          <div className="grid gap-4 sm:grid-cols-1 md:grid-cols-3">
            <div className="flex flex-col items-start gap-4 rounded-lg border p-4">
              <div className="flex h-12 w-12 items-center justify-center rounded-full bg-primary/10">
                <Type className="h-6 w-6 text-primary" />
              </div>
              <h3 className="font-semibold">TypeScript Types</h3>
              <p className="text-sm text-muted-foreground">
                Generate interfaces and types for your API request bodies and responses.
              </p>
              <Button variant="outline" className="mt-auto w-full" onClick={() => handleGenerate('typescript')} disabled={isGenerating || !selectedService}>
                Generate Types
              </Button>
            </div>
            <div className="flex flex-col items-start gap-4 rounded-lg border p-4">
              <div className="flex h-12 w-12 items-center justify-center rounded-full bg-primary/10">
                <Code className="h-6 w-6 text-primary" />
              </div>
              <h3 className="font-semibold">React Query Hooks</h3>
              <p className="text-sm text-muted-foreground">
                Scaffold custom hooks for fetching, creating, and updating data.
              </p>
              <Button variant="outline" className="mt-auto w-full" disabled>Generate Hooks</Button>
            </div>
            <div className="flex flex-col items-start gap-4 rounded-lg border p-4">
              <div className="flex h-12 w-12 items-center justify-center rounded-full bg-primary/10">
                <ToyBrick className="h-6 w-6 text-primary" />
              </div>
              <h3 className="font-semibold">React Components</h3>
              <p className="text-sm text-muted-foreground">
                Create default components for displaying and interacting with your API.
              </p>
              <Button variant="outline" className="mt-auto w-full" disabled>Generate Components</Button>
            </div>
          </div>
        </CardContent>
      </Card>
      <Card className="flex flex-col">
        <CardHeader>
          <CardTitle>Generated Code</CardTitle>
          <CardDescription>The generated client code will appear here.</CardDescription>
        </CardHeader>
        <CardContent className="flex-grow">
          <div className="relative h-full min-h-[300px] rounded-md border bg-secondary/30 p-4">
            {isGenerating && (
              <div className="space-y-2">
                <Skeleton className="h-4 w-3/4" />
                <Skeleton className="h-4 w-1/2" />
                <Skeleton className="h-4 w-5/6" />
                <Skeleton className="h-4 w-2/3" />
              </div>
            )}
            {error && !isGenerating && (
                <div className="flex flex-col items-center justify-center h-full text-center text-destructive">
                    <AlertTriangle className="h-10 w-10 mb-4" />
                    <p className="font-semibold">Generation Failed</p>
                    <p className="text-sm">{error.message}</p>
                </div>
            )}
            {!isGenerating && !generatedCode && !error && (
              <div className="flex items-center justify-center h-full text-center text-muted-foreground">
                <p>Generated code will be displayed here.</p>
              </div>
            )}
            {generatedCode && (
              <>
                <Button
                  variant="ghost"
                  size="icon"
                  className="absolute right-2 top-2 h-8 w-8"
                  onClick={handleCopy}
                >
                  {isCopied ? (
                    <Check className="h-4 w-4 text-accent" />
                  ) : (
                    <Clipboard className="h-4 w-4" />
                  )}
                  <span className="sr-only">Copy to clipboard</span>
                </Button>
                <pre className="h-full w-full overflow-auto whitespace-pre-wrap text-sm">
                  <code>{generatedCode}</code>
                </pre>
              </>
            )}
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
