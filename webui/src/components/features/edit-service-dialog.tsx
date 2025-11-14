
'use client';

import * as React from 'react';
import { useForm } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import { z } from 'zod';
import dynamic from 'next/dynamic';
import * as yaml from 'js-yaml';
import { Loader2 } from 'lucide-react';

// Lazy load Monaco Editor to reduce initial bundle size
const Editor = dynamic(() => import('@monaco-editor/react'), {
  loading: () => (
    <div className="flex items-center justify-center h-[400px] border rounded-md bg-muted">
      <Loader2 className="h-6 w-6 animate-spin text-muted-foreground" />
    </div>
  ),
  ssr: false,
});

import { Button } from '@/components/ui/button';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import { Form, FormControl, FormDescription, FormField, FormItem, FormLabel, FormMessage } from '@/components/ui/form';
import { Input } from '@/components/ui/input';
import { useToast } from '@/hooks/use-toast';
import { updateService, getService } from '@/services/api';
import type { Service } from '@/lib/types';

/**
 * @fileoverview A dialog component for editing an existing service definition with Monaco Editor.
 */

const formSchema = z.object({
  name: z.string().min(3, 'Service name must be at least 3 characters.'),
  version: z.string().regex(/^\d+\.\d+\.\d+$/, 'Version must be in semantic format (e.g., 1.0.0).'),
  port: z.coerce.number().int().min(1024, 'Port must be above 1023.').max(65535, 'Port must be below 65536.'),
  definition: z.string().min(20, 'Service definition must be at least 20 characters.'),
});

type EditServiceFormValues = z.infer<typeof formSchema>;

type EditServiceDialogProps = {
  service: Service;
  onUpdateService: (service: Service) => void;
  onOpenChange: (isOpen: boolean) => void;
};

/**
 * A dialog for editing an existing service definition with Monaco Editor.
 * Features include YAML syntax highlighting, real-time validation, auto-formatting,
 * and unsaved changes warning.
 * @param {EditServiceDialogProps} props - The component props.
 * @returns {React.ReactElement} The rendered EditServiceDialog component.
 */
export function EditServiceDialog({ service, onUpdateService, onOpenChange }: EditServiceDialogProps) {
  const { toast } = useToast();
  const [yamlContent, setYamlContent] = React.useState(service.definition);
  const [validationErrors, setValidationErrors] = React.useState<string[]>([]);
  const [isDirty, setIsDirty] = React.useState(false);
  const [isSaving, setIsSaving] = React.useState(false);
  const [isLoading, setIsLoading] = React.useState(true);
  const editorRef = React.useRef<any>(null);

  const form = useForm<EditServiceFormValues>({
    resolver: zodResolver(formSchema),
    defaultValues: {
      name: service.name,
      version: service.version,
      port: service.port,
      definition: service.definition,
    },
  });

  // Load the full service definition from the backend
  React.useEffect(() => {
    const loadServiceDefinition = async () => {
      try {
        setIsLoading(true);
        const fullService = await getService(service.name);
        
        // Try to reconstruct YAML from the service response
        // If the backend returns the raw YAML, use it directly
        // Otherwise, we'll use the existing definition
        if (fullService && typeof fullService === 'object') {
          // For now, use the existing definition
          // In a real implementation, the backend should return the raw YAML
          setYamlContent(service.definition);
        }
      } catch (error) {
        console.error('Failed to load service:', error);
        toast({
          title: 'Error',
          description: 'Failed to load service definition',
          variant: 'destructive',
        });
      } finally {
        setIsLoading(false);
      }
    };

    loadServiceDefinition();
  }, [service.name, service.definition, toast]);

  // Validate YAML in real-time
  const validateYaml = React.useCallback((content: string) => {
    const errors: string[] = [];
    
    try {
      yaml.load(content);
      setValidationErrors([]);
    } catch (error) {
      if (error instanceof Error) {
        errors.push(error.message);
      } else {
        errors.push('Invalid YAML syntax');
      }
      setValidationErrors(errors);
    }
  }, []);

  // Handle editor content change
  const handleEditorChange = React.useCallback((value: string | undefined) => {
    if (value !== undefined) {
      setYamlContent(value);
      setIsDirty(true);
      form.setValue('definition', value);
      
      // Debounced validation
      const timeoutId = setTimeout(() => {
        validateYaml(value);
      }, 500);
      
      return () => clearTimeout(timeoutId);
    }
  }, [form, validateYaml]);

  // Handle editor mount
  const handleEditorDidMount = (editor: any) => {
    editorRef.current = editor;
    
    // Add custom YAML validation markers
    editor.onDidChangeModelContent(() => {
      const model = editor.getModel();
      if (model && validationErrors.length > 0) {
        // Monaco will show errors in the editor
      }
    });
  };

  // Auto-format YAML
  const handleFormat = React.useCallback(() => {
    try {
      const parsed = yaml.load(yamlContent);
      const formatted = yaml.dump(parsed, {
        indent: 2,
        lineWidth: 80,
        noRefs: true,
      });
      setYamlContent(formatted);
      form.setValue('definition', formatted);
      setIsDirty(true);
      
      toast({
        title: 'Formatted',
        description: 'YAML has been formatted successfully',
      });
    } catch (error) {
      toast({
        title: 'Format Error',
        description: 'Cannot format invalid YAML',
        variant: 'destructive',
      });
    }
  }, [yamlContent, form, toast]);

  // Handle dialog close with unsaved changes warning
  const handleOpenChange = React.useCallback((open: boolean) => {
    if (!open && isDirty) {
      const confirmed = window.confirm(
        'You have unsaved changes. Are you sure you want to close?'
      );
      if (!confirmed) {
        return;
      }
    }
    onOpenChange(open);
  }, [isDirty, onOpenChange]);

  /**
   * Handles form submission to update the service.
   * @param {EditServiceFormValues} values - The updated form values.
   */
  async function onSubmit(values: EditServiceFormValues) {
    // Check for validation errors
    if (validationErrors.length > 0) {
      toast({
        title: 'Validation Error',
        description: 'Please fix YAML errors before saving',
        variant: 'destructive',
      });
      return;
    }

    try {
      setIsSaving(true);
      
      // Call backend API to update service
      await updateService(service.name, values.definition);
      
      // Optimistic update
      onUpdateService({
        ...service,
        ...values,
      });
      
      toast({
        title: 'Success',
        description: `Service "${values.name}" has been updated successfully`,
      });
      
      setIsDirty(false);
      onOpenChange(false);
    } catch (error) {
      console.error('Failed to update service:', error);
      toast({
        title: 'Error',
        description: error instanceof Error ? error.message : 'Failed to update service',
        variant: 'destructive',
      });
    } finally {
      setIsSaving(false);
    }
  }

  return (
    <Dialog open={true} onOpenChange={handleOpenChange}>
      <DialogContent className="sm:max-w-4xl max-h-[90vh] flex flex-col">
        <DialogHeader>
          <DialogTitle>
            Edit Service: {service.name}
            {isDirty && <span className="text-yellow-600 ml-2">●</span>}
          </DialogTitle>
          <DialogDescription>
            Modify the details of your service definition. Changes will be saved to the underlying YAML file.
          </DialogDescription>
        </DialogHeader>
        <Form {...form}>
          <form onSubmit={form.handleSubmit(onSubmit)} className="space-y-4 py-4 flex-1 flex flex-col overflow-hidden">
            <div className="grid grid-cols-1 gap-4 md:grid-cols-3">
              <FormField
                control={form.control}
                name="name"
                render={({ field }) => (
                  <FormItem className="md:col-span-2">
                    <FormLabel>Service Name</FormLabel>
                    <FormControl>
                      <Input placeholder="e.g., Inventory Management" {...field} />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />
              <FormField
                control={form.control}
                name="version"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>Version</FormLabel>
                    <FormControl>
                      <Input placeholder="1.0.0" {...field} />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />
            </div>
            <FormField
              control={form.control}
              name="port"
              render={({ field }) => (
                <FormItem>
                  <FormLabel>Port</FormLabel>
                  <FormControl>
                    <Input type="number" placeholder="3005" {...field} />
                  </FormControl>
                  <FormDescription>The network port the simulated service will run on.</FormDescription>
                  <FormMessage />
                </FormItem>
              )}
            />
            <FormField
              control={form.control}
              name="definition"
              render={() => (
                <FormItem className="flex-1 flex flex-col overflow-hidden">
                  <div className="flex items-center justify-between">
                    <FormLabel>Service Definition (YAML)</FormLabel>
                    <Button
                      type="button"
                      variant="outline"
                      size="sm"
                      onClick={handleFormat}
                      disabled={isLoading || validationErrors.length > 0}
                    >
                      Format
                    </Button>
                  </div>
                  <FormControl>
                    <div className="flex-1 border rounded-md overflow-hidden">
                      {isLoading ? (
                        <div className="flex items-center justify-center h-[400px]">
                          <p className="text-muted-foreground">Loading editor...</p>
                        </div>
                      ) : (
                        <Editor
                          height="400px"
                          defaultLanguage="yaml"
                          value={yamlContent}
                          onChange={handleEditorChange}
                          onMount={handleEditorDidMount}
                          theme="vs-dark"
                          options={{
                            minimap: { enabled: false },
                            lineNumbers: 'on',
                            scrollBeyondLastLine: false,
                            automaticLayout: true,
                            tabSize: 2,
                            wordWrap: 'on',
                            formatOnPaste: true,
                            formatOnType: true,
                          }}
                        />
                      )}
                    </div>
                  </FormControl>
                  {validationErrors.length > 0 && (
                    <div className="text-sm text-destructive mt-2">
                      <p className="font-semibold">Validation Errors:</p>
                      <ul className="list-disc list-inside">
                        {validationErrors.map((error, index) => (
                          <li key={index}>{error}</li>
                        ))}
                      </ul>
                    </div>
                  )}
                  <FormMessage />
                </FormItem>
              )}
            />
            <DialogFooter>
              <Button 
                type="button" 
                variant="ghost" 
                onClick={() => handleOpenChange(false)}
                disabled={isSaving}
              >
                Cancel
              </Button>
              <Button 
                type="submit" 
                disabled={isSaving || validationErrors.length > 0 || isLoading}
              >
                {isSaving ? 'Saving...' : 'Save Changes'}
              </Button>
            </DialogFooter>
          </form>
        </Form>
      </DialogContent>
    </Dialog>
  );
}
