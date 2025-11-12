
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
} from '@/components/ui/dialog';
import { Form, FormControl, FormDescription, FormField, FormItem, FormLabel, FormMessage } from '@/components/ui/form';
import { Input } from '@/components/ui/input';
import { Textarea } from '@/components/ui/textarea';
import type { Service } from '@/lib/types';

/**
 * @fileoverview A dialog component for editing an existing service definition.
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
 * A dialog for editing an existing service definition. It is pre-filled with the
 * service's current data and allows for modification and saving.
 * @param {EditServiceDialogProps} props - The component props.
 * @returns {React.ReactElement} The rendered EditServiceDialog component.
 */
export function EditServiceDialog({ service, onUpdateService, onOpenChange }: EditServiceDialogProps) {
  const form = useForm<EditServiceFormValues>({
    resolver: zodResolver(formSchema),
    defaultValues: {
      name: service.name,
      version: service.version,
      port: service.port,
      definition: service.definition,
    },
  });

  /**
   * Handles form submission to update the service.
   * @param {EditServiceFormValues} values - The updated form values.
   */
  function onSubmit(values: EditServiceFormValues) {
    onUpdateService({
      ...service,
      ...values,
    });
    onOpenChange(false);
  }

  return (
    <Dialog open={true} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-2xl">
        <DialogHeader>
          <DialogTitle>Edit Service: {service.name}</DialogTitle>
          <DialogDescription>
            Modify the details of your service definition. Changes will be saved to the underlying YAML file.
          </DialogDescription>
        </DialogHeader>
        <Form {...form}>
          <form onSubmit={form.handleSubmit(onSubmit)} className="space-y-4 py-4">
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
              render={({ field }) => (
                <FormItem>
                  <FormLabel>Service Definition (YAML)</FormLabel>
                  <FormControl>
                    <Textarea
                      placeholder="Paste your OpenAPI (YAML) or GraphQL schema here."
                      className="min-h-[200px] resize-y font-mono"
                      {...field}
                    />
                  </FormControl>
                  <FormMessage />
                </FormItem>
              )}
            />
            <DialogFooter>
              <Button type="button" variant="ghost" onClick={() => onOpenChange(false)}>
                Cancel
              </Button>
              <Button type="submit">Save Changes</Button>
            </DialogFooter>
          </form>
        </Form>
      </DialogContent>
    </Dialog>
  );
}
