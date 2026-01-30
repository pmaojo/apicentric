'use client';

import * as React from 'react';
import { useForm } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import { z } from 'zod';
import { Sparkles, Clipboard, Check, AlertTriangle, Loader2, Settings } from 'lucide-react';
import yaml from 'js-yaml';

import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Form, FormControl, FormField, FormItem, FormLabel, FormMessage } from '@/components/ui/form';
import { Textarea } from '@/components/ui/textarea';
import { useToast } from '@/hooks/use-toast';
import { Skeleton } from '@/components/ui/skeleton';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { Progress } from '@/components/ui/progress';
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert';
import { aiGenerate, getAiConfig, type AiConfigResponse } from '@/services/api';

const formSchema = z.object({
  prompt: z.string().min(10, {
    message: 'Prompt must be at least 10 characters.',
  }),
  provider: z.enum(['openai', 'gemini', 'local']).optional(),
});

type AiGeneratorProps = {
  onAddService: (service: { name: string, version: string, port: number, definition: string }) => void;
};

export function AiGenerator({ onAddService }: AiGeneratorProps) {
  const { toast } = useToast();
  const [generatedDefinition, setGeneratedDefinition] = React.useState<string | null>(null);
  const [validationErrors, setValidationErrors] = React.useState<string[]>([]);
  const [isLoading, setIsLoading] = React.useState(false);
  const [progress, setProgress] = React.useState(0);
  const [error, setError] = React.useState<string | null>(null);
  const [isCopied, setIsCopied] = React.useState(false);
  const [aiConfig, setAiConfig] = React.useState<AiConfigResponse | null>(null);
  const [showConfigGuide, setShowConfigGuide] = React.useState(false);
  const [isLoadingConfig, setIsLoadingConfig] = React.useState(true);

  const form = useForm<z.infer<typeof formSchema>>({
    resolver: zodResolver(formSchema),
    defaultValues: {
      prompt: '',
      provider: undefined,
    },
  });

  // Load AI configuration on mount
  React.useEffect(() => {
    async function loadAiConfig() {
      setIsLoadingConfig(true);
      try {
        const config = await getAiConfig();
        setAiConfig(config);

        // Set default provider if available
        if (config.provider) {
          form.setValue('provider', config.provider as any);
        }

        // Show config guide if not configured
        if (!config.is_configured) {
          setShowConfigGuide(true);
        }
      } catch (err) {
        console.error('Failed to load AI config:', err);
        setShowConfigGuide(true);
      } finally {
        setIsLoadingConfig(false);
      }
    }
    loadAiConfig();
  }, [form]);

  async function onSubmit(values: z.infer<typeof formSchema>) {
    setIsLoading(true);
    setError(null);
    setGeneratedDefinition(null);
    setValidationErrors([]);
    setProgress(0);
    
    try {
      // Simulate progress for better UX
      const progressInterval = setInterval(() => {
        setProgress(prev => Math.min(prev + 10, 90));
      }, 500);

      const result = await aiGenerate(values.prompt, values.provider);
      
      clearInterval(progressInterval);
      setProgress(100);
      
      setGeneratedDefinition(result.yaml);
      setValidationErrors(result.validation_errors || []);
      
      if (result.validation_errors && result.validation_errors.length > 0) {
        toast({
          variant: 'destructive',
          title: 'Validation Warnings',
          description: `Generated YAML has ${result.validation_errors.length} validation issue(s). Review before applying.`,
        });
      } else {
        toast({
          title: 'Service Generated',
          description: 'AI successfully generated the service definition.',
        });
      }

    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'An unknown error occurred.';
      setError(errorMessage);
      
      // Check if it's a configuration error
      if (errorMessage.includes('AI_NOT_CONFIGURED') || errorMessage.includes('not configured')) {
        setShowConfigGuide(true);
      }
      
      toast({
        variant: 'destructive',
        title: 'Generation Failed',
        description: errorMessage,
      });
    } finally {
      setIsLoading(false);
      setTimeout(() => setProgress(0), 1000);
    }
  }

  function handleApply() {
    if (!generatedDefinition) return;
    
    try {
      const doc: any = yaml.load(generatedDefinition);
      onAddService({
        name: doc.name || 'Untitled AI Service',
        version: doc.version || '1.0.0',
        port: doc.server?.port || 3000,
        definition: generatedDefinition,
      });
      toast({
        title: 'Service Added',
        description: 'The generated service has been added to your services list.',
      });
      
      // Reset form
      form.reset();
      setGeneratedDefinition(null);
      setValidationErrors([]);
    } catch (yamlError) {
      toast({
        variant: 'destructive',
        title: 'Invalid YAML',
        description: 'The generated YAML is invalid. Please review and fix errors.',
      });
    }
  }

  const handleCopy = () => {
    if (generatedDefinition) {
      navigator.clipboard.writeText(generatedDefinition);
      setIsCopied(true);
      toast({ title: 'Copied to clipboard!' });
      setTimeout(() => setIsCopied(false), 2000);
    }
  };

  return (
    <div className="space-y-6">
      {/* AI Configuration Status */}
      {showConfigGuide && (
        <Alert>
          <Settings className="h-4 w-4" />
          <AlertTitle>AI Configuration Required</AlertTitle>
          <AlertDescription>
            <p className="mb-2">
              AI generation requires configuration. Please set up one of the following providers:
            </p>
            <ul className="list-disc list-inside space-y-1 text-sm">
              <li><strong>OpenAI:</strong> Set <code>OPENAI_API_KEY</code> environment variable</li>
              <li><strong>Gemini:</strong> Set <code>GEMINI_API_KEY</code> environment variable</li>
              <li><strong>Local:</strong> Configure local LLM endpoint in settings</li>
            </ul>
            <Button 
              variant="outline" 
              size="sm" 
              className="mt-3"
              onClick={() => setShowConfigGuide(false)}
            >
              Dismiss
            </Button>
          </AlertDescription>
        </Alert>
      )}

      <div className="grid grid-cols-1 gap-8 lg:grid-cols-2">
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <Sparkles className="h-6 w-6 text-primary" />
              <span>Describe Your Service</span>
            </CardTitle>
            <CardDescription>
              Use natural language to describe the API you want to create. Provide details about endpoints, data models, and desired functionality.
            </CardDescription>
          </CardHeader>
          <CardContent>
            <Form {...form}>
              <form onSubmit={form.handleSubmit(onSubmit)} className="space-y-6">
                <FormField
                  control={form.control}
                  name="provider"
                  render={({ field }) => (
                    <FormItem>
                      <FormLabel>AI Provider</FormLabel>
                      <Select 
                        onValueChange={field.onChange} 
                        value={field.value}
                        disabled={isLoadingConfig || isLoading}
                      >
                        <FormControl>
                          <SelectTrigger>
                            <SelectValue placeholder="Select AI provider" />
                          </SelectTrigger>
                        </FormControl>
                        <SelectContent>
                          <SelectItem value="openai">OpenAI</SelectItem>
                          <SelectItem value="gemini">Google Gemini</SelectItem>
                          <SelectItem value="local">Local LLM</SelectItem>
                          {(!aiConfig || !aiConfig.is_configured) && (
                            <SelectItem value="none" disabled>No providers configured</SelectItem>
                          )}
                        </SelectContent>
                      </Select>
                      <FormMessage />
                    </FormItem>
                  )}
                />
                
                <FormField
                  control={form.control}
                  name="prompt"
                  render={({ field }) => (
                    <FormItem>
                      <FormLabel>Service Prompt</FormLabel>
                      <FormControl>
                        <Textarea
                          placeholder="e.g., 'Create a simple user management API with endpoints for creating, reading, updating, and deleting users. A user should have an id, name, and email.'"
                          className="min-h-[200px] resize-y"
                          disabled={isLoading}
                          {...field}
                        />
                      </FormControl>
                      <FormMessage />
                    </FormItem>
                  )}
                />

                {isLoading && (
                  <div className="space-y-2">
                    <div className="flex items-center justify-between text-sm text-muted-foreground">
                      <span>Generating service definition...</span>
                      <span>{progress}%</span>
                    </div>
                    <Progress value={progress} className="h-2" />
                  </div>
                )}
                
                <Button 
                  type="submit" 
                  disabled={isLoading || !aiConfig?.is_configured}
                >
                  {isLoading ? (
                    <>
                      <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                      Generating...
                    </>
                  ) : (
                    <>
                      Generate Definition
                      <Sparkles className="ml-2 h-4 w-4" />
                    </>
                  )}
                </Button>
              </form>
            </Form>
          </CardContent>
        </Card>
        <Card className="flex flex-col">
          <CardHeader>
            <CardTitle>Generated Service Definition</CardTitle>
            <CardDescription>
              The AI-generated YAML definition will appear here. Review validation errors and apply when ready.
            </CardDescription>
          </CardHeader>
          <CardContent className="flex-grow space-y-4">
            {/* Validation Errors */}
            {validationErrors.length > 0 && (
              <Alert variant="destructive">
                <AlertTriangle className="h-4 w-4" />
                <AlertTitle>Validation Issues ({validationErrors.length})</AlertTitle>
                <AlertDescription>
                  <ul className="list-disc list-inside space-y-1 text-sm mt-2">
                    {validationErrors.slice(0, 5).map((error, index) => (
                      <li key={index}>{error}</li>
                    ))}
                    {validationErrors.length > 5 && (
                      <li className="text-muted-foreground">
                        ...and {validationErrors.length - 5} more
                      </li>
                    )}
                  </ul>
                </AlertDescription>
              </Alert>
            )}

            {/* YAML Preview */}
            <div className="relative h-full min-h-[300px] rounded-md border bg-secondary/30 p-4">
              {isLoading && (
                <div className="space-y-2">
                  <Skeleton className="h-4 w-3/4" />
                  <Skeleton className="h-4 w-1/2" />
                  <Skeleton className="h-4 w-5/6" />
                  <Skeleton className="h-4 w-2/3" />
                  <Skeleton className="h-4 w-4/5" />
                  <Skeleton className="h-4 w-3/5" />
                </div>
              )}
              {error && !isLoading && (
                <div className="flex flex-col items-center justify-center h-full text-center text-destructive">
                  <AlertTriangle className="h-10 w-10 mb-4" />
                  <p className="font-semibold">Generation Failed</p>
                  <p className="text-sm">{error}</p>
                </div>
              )}
              {!isLoading && !error && !generatedDefinition && (
                <div className="flex items-center justify-center h-full text-center text-muted-foreground">
                  <p>Your generated definition will be displayed here.</p>
                </div>
              )}
              {generatedDefinition && (
                <>
                  <Button
                    variant="ghost"
                    size="icon"
                    className="absolute right-2 top-2 h-8 w-8 z-10"
                    onClick={handleCopy}
                  >
                    {isCopied ? (
                      <Check className="h-4 w-4 text-accent" />
                    ) : (
                      <Clipboard className="h-4 w-4" />
                    )}
                    <span className="sr-only">Copy to clipboard</span>
                  </Button>
                  <pre className="h-full w-full overflow-auto whitespace-pre-wrap text-sm font-mono">
                    <code className="language-yaml">{generatedDefinition}</code>
                  </pre>
                </>
              )}
            </div>

            {/* Action Buttons */}
            {generatedDefinition && (
              <div className="flex gap-2 justify-end">
                <Button
                  variant="outline"
                  onClick={() => {
                    setGeneratedDefinition(null);
                    setValidationErrors([]);
                  }}
                >
                  Clear
                </Button>
                <Button
                  onClick={handleApply}
                  disabled={validationErrors.length > 0}
                >
                  <Check className="mr-2 h-4 w-4" />
                  Apply Service
                </Button>
              </div>
            )}
          </CardContent>
        </Card>
      </div>
    </div>
  );
}
