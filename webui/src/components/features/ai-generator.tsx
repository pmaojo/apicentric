'use client';

import * as React from 'react';
import { useForm } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import { z } from 'zod';
import { Sparkles, Clipboard, Check, AlertTriangle } from 'lucide-react';
import yaml from 'js-yaml';

import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Form, FormControl, FormField, FormItem, FormLabel, FormMessage } from '@/components/ui/form';
import { Textarea } from '@/components/ui/textarea';
import { useToast } from '@/hooks/use-toast';
import { generateServiceDefinitionAction } from '@/app/actions';
import { Skeleton } from '@/components/ui/skeleton';

const formSchema = z.object({
  prompt: z.string().min(10, {
    message: 'Prompt must be at least 10 characters.',
  }),
});

type AiGeneratorProps = {
  onAddService: (service: { name: string, version: string, port: number, definition: string }) => void;
};

export function AiGenerator({ onAddService }: AiGeneratorProps) {
  const { toast } = useToast();
  const [generatedDefinition, setGeneratedDefinition] = React.useState<string | null>(null);
  const [isLoading, setIsLoading] = React.useState(false);
  const [error, setError] = React.useState<string | null>(null);
  const [isCopied, setIsCopied] = React.useState(false);

  const form = useForm<z.infer<typeof formSchema>>({
    resolver: zodResolver(formSchema),
    defaultValues: {
      prompt: '',
    },
  });

  async function onSubmit(values: z.infer<typeof formSchema>) {
    setIsLoading(true);
    setError(null);
    setGeneratedDefinition(null);
    try {
      const result = await generateServiceDefinitionAction(values.prompt);
      setGeneratedDefinition(result);
      
      // Auto-add the service to the list
      try {
        const doc: any = yaml.load(result);
        onAddService({
          name: doc.name || 'Untitled AI Service',
          version: doc.version || '1.0.0',
          port: doc.server?.port || 3000,
          definition: result,
        });
        toast({
            title: 'Service Generated & Added',
            description: 'The new service definition has been added to your services list.',
        });
      } catch (yamlError) {
          toast({
            variant: 'destructive',
            title: 'Invalid YAML Generated',
            description: 'The AI produced invalid YAML. Please try a different prompt.',
          });
      }

    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'An unknown error occurred.';
      setError(errorMessage);
      toast({
        variant: 'destructive',
        title: 'Generation Failed',
        description: errorMessage,
      });
    } finally {
      setIsLoading(false);
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
                name="prompt"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>Service Prompt</FormLabel>
                    <FormControl>
                      <Textarea
                        placeholder="e.g., 'Create a simple user management API with endpoints for creating, reading, updating, and deleting users. A user should have an id, name, and email.'"
                        className="min-h-[200px] resize-y"
                        {...field}
                      />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />
              <Button type="submit" disabled={isLoading}>
                {isLoading ? 'Generating...' : 'Generate Definition'}
                <Sparkles className="ml-2 h-4 w-4" />
              </Button>
            </form>
          </Form>
        </CardContent>
      </Card>
      <Card className="flex flex-col">
        <CardHeader>
          <CardTitle>Generated Service Definition</CardTitle>
          <CardDescription>
            The AI-generated YAML or GraphQL definition will appear here. Review and edit as needed.
          </CardDescription>
        </CardHeader>
        <CardContent className="flex-grow">
          <div className="relative h-full min-h-[300px] rounded-md border bg-secondary/30 p-4">
            {isLoading && (
              <div className="space-y-2">
                <Skeleton className="h-4 w-3/4" />
                <Skeleton className="h-4 w-1/2" />
                <Skeleton className="h-4 w-5/6" />
                <Skeleton className="h-4 w-2/3" />
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
                  <code>{generatedDefinition}</code>
                </pre>
              </>
            )}
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
