
'use client';

import * as React from 'react';
import { useForm } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import { z } from 'zod';

import { Button } from '@/components/ui/button';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from '@/components/ui/dialog';
import { Form, FormControl, FormField, FormItem, FormLabel, FormMessage } from '@/components/ui/form';
import { Input } from '@/components/ui/input';
import { useToast } from '@/hooks/use-toast';
import { createGraphQLService } from '@/services/api';
import { Loader2 } from 'lucide-react';
import type { Service } from '@/lib/types';

/**
 * @fileoverview A dialog component for creating new GraphQL services.
 * It provides a form for specifying the service name and port, and then
 * calls an API to generate the necessary starter files.
 */

const formSchema = z.object({
  name: z.string().min(3, 'Service name must be at least 3 characters.').regex(/^[a-z0-9-]+$/, 'Name can only contain lowercase letters, numbers, and hyphens.'),
  port: z.coerce.number().int().min(1024, 'Port must be above 1023.').max(65535, 'Port must be below 65536.'),
});

type CreateGraphQLServiceFormValues = z.infer<typeof formSchema>;

type CreateGraphQLServiceDialogProps = {
  children: React.ReactNode;
  onAddService: (service: Service) => void;
};

/**
 * A dialog for creating a new GraphQL service.
 * @param {CreateGraphQLServiceDialogProps} props - The component props.
 * @returns {React.ReactElement} The rendered CreateGraphQLServiceDialog component.
 */
export function CreateGraphQLServiceDialog({ children, onAddService }: CreateGraphQLServiceDialogProps) {
  const [isOpen, setIsOpen] = React.useState(false);
  const [isCreating, setIsCreating] = React.useState(false);
  const { toast } = useToast();

  const form = useForm<CreateGraphQLServiceFormValues>({
    resolver: zodResolver(formSchema),
    defaultValues: {
      name: '',
      port: 3006,
    },
  });

  /**
   * Handles form submission to create the new GraphQL service files.
   * @param {CreateGraphQLServiceFormValues} values - The form values.
   */
  async function onSubmit(values: CreateGraphQLServiceFormValues) {
    setIsCreating(true);
    try {
      const newService = await createGraphQLService(values.name, values.port);
      onAddService(newService);
      setIsOpen(false);
      form.reset();
    } catch (error) {
      toast({
        variant: 'destructive',
        title: 'Failed to create GraphQL service',
        description: error instanceof Error ? error.message : 'An unknown error occurred.',
      });
    } finally {
        setIsCreating(false);
    }
  }

  return (
    <Dialog open={isOpen} onOpenChange={setIsOpen}>
      <DialogTrigger asChild>{children}</DialogTrigger>
      <DialogContent className="sm:max-w-lg">
        <DialogHeader>
          <DialogTitle>Create New GraphQL Service</DialogTitle>
          <DialogDescription>
            This will generate a starter set of files for a GraphQL service, including a schema and a mock response.
          </DialogDescription>
        </DialogHeader>
        <Form {...form}>
          <form onSubmit={form.handleSubmit(onSubmit)} className="space-y-4 py-4">
            <FormField
              control={form.control}
              name="name"
              render={({ field }) => (
                <FormItem>
                  <FormLabel>Service Name</FormLabel>
                  <FormControl>
                    <Input placeholder="e.g., product-api" {...field} />
                  </FormControl>
                  <FormMessage />
                </FormItem>
              )}
            />
            <FormField
              control={form.control}
              name="port"
              render={({ field }) => (
                <FormItem>
                  <FormLabel>Port</FormLabel>
                  <FormControl>
                    <Input type="number" placeholder="3006" {...field} />
                  </FormControl>
                  <FormMessage />
                </FormItem>
              )}
            />
            <DialogFooter>
              <Button type="button" variant="ghost" onClick={() => setIsOpen(false)} disabled={isCreating}>
                Cancel
              </Button>
              <Button type="submit" disabled={isCreating}>
                {isCreating && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
                {isCreating ? 'Creating...' : 'Create Service'}
              </Button>
            </DialogFooter>
          </form>
        </Form>
      </DialogContent>
    </Dialog>
  );
}
