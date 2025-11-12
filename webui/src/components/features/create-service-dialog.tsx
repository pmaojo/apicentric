
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
import { Form, FormControl, FormDescription, FormField, FormItem, FormLabel, FormMessage } from '@/components/ui/form';
import { Input } from '@/components/ui/input';
import { Textarea } from '@/components/ui/textarea';

/**
 * @fileoverview Provides a dialog for creating a new REST service.
 */

const formSchema = z.object({
  name: z.string().min(3, 'Service name must be at least 3 characters.'),
  version: z.string().regex(/^\d+\.\d+\.\d+$/, 'Version must be in semantic format (e.g., 1.0.0).'),
  port: z.coerce.number().int().min(1024, 'Port must be above 1023.').max(65535, 'Port must be below 65536.'),
  definition: z.string().min(20, 'Service definition must be at least 20 characters.'),
});

type CreateServiceFormValues = z.infer<typeof formSchema>;

type CreateServiceDialogProps = {
  children: React.ReactNode;
  onAddService: (service: CreateServiceFormValues) => void;
};

/**
 * A dialog component that allows users to create a new REST service
 * by providing its definition and configuration.
 * @param {CreateServiceDialogProps} props - The component props.
 * @returns {React.ReactElement} The rendered dialog component.
 */
export function CreateServiceDialog({ children, onAddService }: CreateServiceDialogProps) {
  const [isOpen, setIsOpen] = React.useState(false);

  const form = useForm<CreateServiceFormValues>({
    resolver: zodResolver(formSchema),
    defaultValues: {
      name: '',
      version: '1.0.0',
      port: 3005,
      definition: '',
    },
  });

  /**
   * Handles form submission, calling the onAddService callback and closing the dialog.
   * @param {CreateServiceFormValues} values - The form values.
   */
  function onSubmit(values: CreateServiceFormValues) {
    onAddService(values);
    setIsOpen(false);
    form.reset();
  }

  return (
    <Dialog open={isOpen} onOpenChange={setIsOpen}>
      <DialogTrigger asChild>{children}</DialogTrigger>
      <DialogContent className="sm:max-w-2xl">
        <DialogHeader>
          <DialogTitle>Create New Service Definition</DialogTitle>
          <DialogDescription>
            Define a new service by providing its details below. This will add it to your list of manageable services.
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
                  <FormLabel>Service Definition (OpenAPI/GraphQL)</FormLabel>
                  <FormControl>
                    <Textarea
                      placeholder="Paste your OpenAPI (YAML/JSON) or GraphQL schema here."
                      className="min-h-[200px] resize-y font-mono"
                      {...field}
                    />
                  </FormControl>
                  <FormMessage />
                </FormItem>
              )}
            />
            <DialogFooter>
              <Button type="button" variant="ghost" onClick={() => setIsOpen(false)}>
                Cancel
              </Button>
              <Button type="submit">Create Service</Button>
            </DialogFooter>
          </form>
        </Form>
      </DialogContent>
    </Dialog>
  );
}
