'use client';

import * as React from 'react';
import { useForm } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import * as z from 'zod';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Switch } from '@/components/ui/switch';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import {
  Form,
  FormControl,
  FormDescription,
  FormField,
  FormItem,
  FormLabel,
  FormMessage,
} from '@/components/ui/form';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
import { useToast } from '@/hooks/use-toast';
import { getConfig, updateConfig, validateConfig } from '@/services/api';
import { Loader2, Eye, EyeOff, Save, RotateCcw, AlertCircle } from 'lucide-react';
import { Alert, AlertDescription } from '@/components/ui/alert';

// Configuration schema with validation
const configSchema = z.object({
  // General settings
  general: z.object({
    host: z.string().default('0.0.0.0'),
    port: z.number().min(1).max(65535).default(8080),
    services_directory: z.string().default('./services'),
    protect_services: z.boolean().default(false),
  }),
  
  // AI settings
  ai: z.object({
    enabled: z.boolean().default(false),
    provider: z.enum(['openai', 'gemini', 'local']).default('openai'),
    openai_api_key: z.string().optional(),
    openai_model: z.string().default('gpt-4'),
    gemini_api_key: z.string().optional(),
    gemini_model: z.string().default('gemini-pro'),
    local_endpoint: z.string().url().optional(),
  }),
  
  // Services settings
  services: z.object({
    auto_reload: z.boolean().default(true),
    default_port_range_start: z.number().min(1024).max(65535).default(8000),
    default_port_range_end: z.number().min(1024).max(65535).default(9000),
    enable_cors: z.boolean().default(true),
  }),
  
  // Advanced settings
  advanced: z.object({
    log_level: z.enum(['error', 'warn', 'info', 'debug', 'trace']).default('info'),
    max_log_entries: z.number().min(100).max(100000).default(10000),
    jwt_secret: z.string().optional(),
    jwt_expiry_hours: z.number().min(1).max(720).default(24),
    allowed_origins: z.string().default('*'),
    enable_websocket: z.boolean().default(true),
  }),
});

type ConfigFormValues = z.infer<typeof configSchema>;

const transformBackendToForm = (config: Record<string, any>): ConfigFormValues => {
  // Transform backend configuration to form structure
  return {
    general: {
      host: config.host || '0.0.0.0',
      port: config.port || 8080,
      services_directory: config.services_directory || './services',
      protect_services: config.protect_services || false,
    },
    ai: {
      enabled: config.ai_enabled || false,
      provider: config.ai_provider || 'openai',
      openai_api_key: config.openai_api_key,
      openai_model: config.openai_model || 'gpt-4',
      gemini_api_key: config.gemini_api_key,
      gemini_model: config.gemini_model || 'gemini-pro',
      local_endpoint: config.local_ai_endpoint,
    },
    services: {
      auto_reload: config.auto_reload !== false,
      default_port_range_start: config.default_port_range_start || 8000,
      default_port_range_end: config.default_port_range_end || 9000,
      enable_cors: config.enable_cors !== false,
    },
    advanced: {
      log_level: config.log_level || 'info',
      max_log_entries: config.max_log_entries || 10000,
      jwt_secret: config.jwt_secret,
      jwt_expiry_hours: config.jwt_expiry_hours || 24,
      allowed_origins: config.allowed_origins || '*',
      enable_websocket: config.enable_websocket !== false,
    },
  };
};

const transformFormToBackend = (values: ConfigFormValues): Record<string, any> => {
  // Transform form values to backend configuration format
  return {
    host: values.general.host,
    port: values.general.port,
    services_directory: values.general.services_directory,
    protect_services: values.general.protect_services,
    ai_enabled: values.ai.enabled,
    ai_provider: values.ai.provider,
    openai_api_key: values.ai.openai_api_key,
    openai_model: values.ai.openai_model,
    gemini_api_key: values.ai.gemini_api_key,
    gemini_model: values.ai.gemini_model,
    local_ai_endpoint: values.ai.local_endpoint,
    auto_reload: values.services.auto_reload,
    default_port_range_start: values.services.default_port_range_start,
    default_port_range_end: values.services.default_port_range_end,
    enable_cors: values.services.enable_cors,
    log_level: values.advanced.log_level,
    max_log_entries: values.advanced.max_log_entries,
    jwt_secret: values.advanced.jwt_secret,
    jwt_expiry_hours: values.advanced.jwt_expiry_hours,
    allowed_origins: values.advanced.allowed_origins,
    enable_websocket: values.advanced.enable_websocket,
  };
};

export function Configuration() {
  const { toast } = useToast();
  const [loading, setLoading] = React.useState(true);
  const [saving, setSaving] = React.useState(false);
  const [validationErrors, setValidationErrors] = React.useState<string[]>([]);
  const [revealedFields, setRevealedFields] = React.useState<Set<string>>(new Set());
  const [originalConfig, setOriginalConfig] = React.useState<ConfigFormValues | null>(null);

  const form = useForm<ConfigFormValues>({
    resolver: zodResolver(configSchema),
    defaultValues: {
      general: {
        host: '0.0.0.0',
        port: 8080,
        services_directory: './services',
        protect_services: false,
      },
      ai: {
        enabled: false,
        provider: 'openai',
        openai_model: 'gpt-4',
        gemini_model: 'gemini-pro',
      },
      services: {
        auto_reload: true,
        default_port_range_start: 8000,
        default_port_range_end: 9000,
        enable_cors: true,
      },
      advanced: {
        log_level: 'info',
        max_log_entries: 10000,
        jwt_expiry_hours: 24,
        allowed_origins: '*',
        enable_websocket: true,
      },
    },
  });

  // Load configuration on mount
  React.useEffect(() => {
    const loadConfiguration = async () => {
      setLoading(true);
      try {
        const config = await getConfig();

        // Transform backend config to form values
        const formValues = transformBackendToForm(config);
        form.reset(formValues);
        setOriginalConfig(formValues);

        toast({
          title: 'Configuration Loaded',
          description: 'Current configuration has been loaded successfully.',
        });
      } catch (error) {
        toast({
          variant: 'destructive',
          title: 'Failed to Load Configuration',
          description: error instanceof Error ? error.message : 'An unknown error occurred',
        });
      } finally {
        setLoading(false);
      }
    };
    loadConfiguration();
  }, [form, toast]);

  const onSubmit = async (values: ConfigFormValues) => {
    setSaving(true);
    setValidationErrors([]);
    
    try {
      // Transform form values to backend config format
      const backendConfig = transformFormToBackend(values);
      
      // Validate configuration
      const validation = await validateConfig(backendConfig);
      
      if (!validation.valid) {
        setValidationErrors(validation.errors);
        toast({
          variant: 'destructive',
          title: 'Validation Failed',
          description: 'Please fix the errors before saving.',
        });
        return;
      }
      
      // Update configuration
      await updateConfig(backendConfig);
      setOriginalConfig(values);
      
      toast({
        title: 'Configuration Saved',
        description: 'Configuration has been updated successfully. Some changes may require a restart.',
      });
    } catch (error) {
      toast({
        variant: 'destructive',
        title: 'Failed to Save Configuration',
        description: error instanceof Error ? error.message : 'An unknown error occurred',
      });
    } finally {
      setSaving(false);
    }
  };

  const handleReset = () => {
    if (originalConfig) {
      form.reset(originalConfig);
      setValidationErrors([]);
      toast({
        title: 'Configuration Reset',
        description: 'Changes have been discarded.',
      });
    }
  };

  const toggleFieldVisibility = (fieldName: string) => {
    setRevealedFields((prev) => {
      const next = new Set(prev);
      if (next.has(fieldName)) {
        next.delete(fieldName);
      } else {
        next.add(fieldName);
      }
      return next;
    });
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center h-96">
        <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
      </div>
    );
  }

  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-3xl font-bold tracking-tight">Configuration</h2>
        <p className="text-muted-foreground">
          Manage simulator settings and preferences
        </p>
      </div>

      {validationErrors.length > 0 && (
        <Alert variant="destructive">
          <AlertCircle className="h-4 w-4" />
          <AlertDescription>
            <div className="font-semibold mb-2">Validation Errors:</div>
            <ul className="list-disc list-inside space-y-1">
              {validationErrors.map((error, index) => (
                <li key={index} className="text-sm">{error}</li>
              ))}
            </ul>
          </AlertDescription>
        </Alert>
      )}

      <Form {...form}>
        <form onSubmit={form.handleSubmit(onSubmit)} className="space-y-6">
          <Tabs defaultValue="general" className="w-full">
            <TabsList className="grid w-full grid-cols-4">
              <TabsTrigger value="general">General</TabsTrigger>
              <TabsTrigger value="ai">AI</TabsTrigger>
              <TabsTrigger value="services">Services</TabsTrigger>
              <TabsTrigger value="advanced">Advanced</TabsTrigger>
            </TabsList>

            {/* General Tab */}
            <TabsContent value="general" className="space-y-4">
              <Card>
                <CardHeader>
                  <CardTitle>General Settings</CardTitle>
                  <CardDescription>
                    Configure basic simulator settings
                  </CardDescription>
                </CardHeader>
                <CardContent className="space-y-4">
                  <FormField
                    control={form.control}
                    name="general.host"
                    render={({ field }) => (
                      <FormItem>
                        <FormLabel>Host</FormLabel>
                        <FormControl>
                          <Input placeholder="0.0.0.0" {...field} />
                        </FormControl>
                        <FormDescription>
                          The host address to bind the server to
                        </FormDescription>
                        <FormMessage />
                      </FormItem>
                    )}
                  />

                  <FormField
                    control={form.control}
                    name="general.port"
                    render={({ field }) => (
                      <FormItem>
                        <FormLabel>Port</FormLabel>
                        <FormControl>
                          <Input
                            type="number"
                            placeholder="8080"
                            {...field}
                            onChange={(e) => field.onChange(parseInt(e.target.value))}
                          />
                        </FormControl>
                        <FormDescription>
                          The port number for the simulator server
                        </FormDescription>
                        <FormMessage />
                      </FormItem>
                    )}
                  />

                  <FormField
                    control={form.control}
                    name="general.services_directory"
                    render={({ field }) => (
                      <FormItem>
                        <FormLabel>Services Directory</FormLabel>
                        <FormControl>
                          <Input placeholder="./services" {...field} />
                        </FormControl>
                        <FormDescription>
                          Directory where service definitions are stored
                        </FormDescription>
                        <FormMessage />
                      </FormItem>
                    )}
                  />

                  <FormField
                    control={form.control}
                    name="general.protect_services"
                    render={({ field }) => (
                      <FormItem className="flex flex-row items-center justify-between rounded-lg border p-4">
                        <div className="space-y-0.5">
                          <FormLabel className="text-base">
                            Protect Services
                          </FormLabel>
                          <FormDescription>
                            Require authentication to access service management endpoints
                          </FormDescription>
                        </div>
                        <FormControl>
                          <Switch
                            checked={field.value}
                            onCheckedChange={field.onChange}
                          />
                        </FormControl>
                      </FormItem>
                    )}
                  />
                </CardContent>
              </Card>
            </TabsContent>

            {/* AI Tab */}
            <TabsContent value="ai" className="space-y-4">
              <Card>
                <CardHeader>
                  <CardTitle>AI Configuration</CardTitle>
                  <CardDescription>
                    Configure AI providers for service generation
                  </CardDescription>
                </CardHeader>
                <CardContent className="space-y-4">
                  <FormField
                    control={form.control}
                    name="ai.enabled"
                    render={({ field }) => (
                      <FormItem className="flex flex-row items-center justify-between rounded-lg border p-4">
                        <div className="space-y-0.5">
                          <FormLabel className="text-base">
                            Enable AI Generation
                          </FormLabel>
                          <FormDescription>
                            Enable AI-powered service definition generation
                          </FormDescription>
                        </div>
                        <FormControl>
                          <Switch
                            checked={field.value}
                            onCheckedChange={field.onChange}
                          />
                        </FormControl>
                      </FormItem>
                    )}
                  />

                  <FormField
                    control={form.control}
                    name="ai.provider"
                    render={({ field }) => (
                      <FormItem>
                        <FormLabel>AI Provider</FormLabel>
                        <Select
                          onValueChange={field.onChange}
                          defaultValue={field.value}
                        >
                          <FormControl>
                            <SelectTrigger>
                              <SelectValue placeholder="Select a provider" />
                            </SelectTrigger>
                          </FormControl>
                          <SelectContent>
                            <SelectItem value="openai">OpenAI</SelectItem>
                            <SelectItem value="gemini">Google Gemini</SelectItem>
                            <SelectItem value="local">Local Model</SelectItem>
                          </SelectContent>
                        </Select>
                        <FormDescription>
                          Choose the AI provider for generation
                        </FormDescription>
                        <FormMessage />
                      </FormItem>
                    )}
                  />

                  {form.watch('ai.provider') === 'openai' && (
                    <>
                      <FormField
                        control={form.control}
                        name="ai.openai_api_key"
                        render={({ field }) => (
                          <FormItem>
                            <FormLabel>OpenAI API Key</FormLabel>
                            <div className="flex gap-2">
                              <FormControl>
                                <Input
                                  type={revealedFields.has('openai_api_key') ? 'text' : 'password'}
                                  placeholder="sk-..."
                                  {...field}
                                />
                              </FormControl>
                              <Button
                                type="button"
                                variant="outline"
                                size="icon"
                                onClick={() => toggleFieldVisibility('openai_api_key')}
                              >
                                {revealedFields.has('openai_api_key') ? (
                                  <EyeOff className="h-4 w-4" />
                                ) : (
                                  <Eye className="h-4 w-4" />
                                )}
                              </Button>
                            </div>
                            <FormDescription>
                              Your OpenAI API key for GPT models
                            </FormDescription>
                            <FormMessage />
                          </FormItem>
                        )}
                      />

                      <FormField
                        control={form.control}
                        name="ai.openai_model"
                        render={({ field }) => (
                          <FormItem>
                            <FormLabel>OpenAI Model</FormLabel>
                            <FormControl>
                              <Input placeholder="gpt-4" {...field} />
                            </FormControl>
                            <FormDescription>
                              The OpenAI model to use (e.g., gpt-4, gpt-3.5-turbo)
                            </FormDescription>
                            <FormMessage />
                          </FormItem>
                        )}
                      />
                    </>
                  )}

                  {form.watch('ai.provider') === 'gemini' && (
                    <>
                      <FormField
                        control={form.control}
                        name="ai.gemini_api_key"
                        render={({ field }) => (
                          <FormItem>
                            <FormLabel>Gemini API Key</FormLabel>
                            <div className="flex gap-2">
                              <FormControl>
                                <Input
                                  type={revealedFields.has('gemini_api_key') ? 'text' : 'password'}
                                  placeholder="AIza..."
                                  {...field}
                                />
                              </FormControl>
                              <Button
                                type="button"
                                variant="outline"
                                size="icon"
                                onClick={() => toggleFieldVisibility('gemini_api_key')}
                              >
                                {revealedFields.has('gemini_api_key') ? (
                                  <EyeOff className="h-4 w-4" />
                                ) : (
                                  <Eye className="h-4 w-4" />
                                )}
                              </Button>
                            </div>
                            <FormDescription>
                              Your Google Gemini API key
                            </FormDescription>
                            <FormMessage />
                          </FormItem>
                        )}
                      />

                      <FormField
                        control={form.control}
                        name="ai.gemini_model"
                        render={({ field }) => (
                          <FormItem>
                            <FormLabel>Gemini Model</FormLabel>
                            <FormControl>
                              <Input placeholder="gemini-pro" {...field} />
                            </FormControl>
                            <FormDescription>
                              The Gemini model to use
                            </FormDescription>
                            <FormMessage />
                          </FormItem>
                        )}
                      />
                    </>
                  )}

                  {form.watch('ai.provider') === 'local' && (
                    <FormField
                      control={form.control}
                      name="ai.local_endpoint"
                      render={({ field }) => (
                        <FormItem>
                          <FormLabel>Local Model Endpoint</FormLabel>
                          <FormControl>
                            <Input
                              type="url"
                              placeholder="http://localhost:11434"
                              {...field}
                            />
                          </FormControl>
                          <FormDescription>
                            URL of your local AI model endpoint (e.g., Ollama)
                          </FormDescription>
                          <FormMessage />
                        </FormItem>
                      )}
                    />
                  )}
                </CardContent>
              </Card>
            </TabsContent>

            {/* Services Tab */}
            <TabsContent value="services" className="space-y-4">
              <Card>
                <CardHeader>
                  <CardTitle>Service Settings</CardTitle>
                  <CardDescription>
                    Configure service management behavior
                  </CardDescription>
                </CardHeader>
                <CardContent className="space-y-4">
                  <FormField
                    control={form.control}
                    name="services.auto_reload"
                    render={({ field }) => (
                      <FormItem className="flex flex-row items-center justify-between rounded-lg border p-4">
                        <div className="space-y-0.5">
                          <FormLabel className="text-base">
                            Auto Reload
                          </FormLabel>
                          <FormDescription>
                            Automatically reload services when definition files change
                          </FormDescription>
                        </div>
                        <FormControl>
                          <Switch
                            checked={field.value}
                            onCheckedChange={field.onChange}
                          />
                        </FormControl>
                      </FormItem>
                    )}
                  />

                  <FormField
                    control={form.control}
                    name="services.default_port_range_start"
                    render={({ field }) => (
                      <FormItem>
                        <FormLabel>Default Port Range Start</FormLabel>
                        <FormControl>
                          <Input
                            type="number"
                            placeholder="8000"
                            {...field}
                            onChange={(e) => field.onChange(parseInt(e.target.value))}
                          />
                        </FormControl>
                        <FormDescription>
                          Starting port for automatic port assignment
                        </FormDescription>
                        <FormMessage />
                      </FormItem>
                    )}
                  />

                  <FormField
                    control={form.control}
                    name="services.default_port_range_end"
                    render={({ field }) => (
                      <FormItem>
                        <FormLabel>Default Port Range End</FormLabel>
                        <FormControl>
                          <Input
                            type="number"
                            placeholder="9000"
                            {...field}
                            onChange={(e) => field.onChange(parseInt(e.target.value))}
                          />
                        </FormControl>
                        <FormDescription>
                          Ending port for automatic port assignment
                        </FormDescription>
                        <FormMessage />
                      </FormItem>
                    )}
                  />

                  <FormField
                    control={form.control}
                    name="services.enable_cors"
                    render={({ field }) => (
                      <FormItem className="flex flex-row items-center justify-between rounded-lg border p-4">
                        <div className="space-y-0.5">
                          <FormLabel className="text-base">
                            Enable CORS
                          </FormLabel>
                          <FormDescription>
                            Enable Cross-Origin Resource Sharing for all services
                          </FormDescription>
                        </div>
                        <FormControl>
                          <Switch
                            checked={field.value}
                            onCheckedChange={field.onChange}
                          />
                        </FormControl>
                      </FormItem>
                    )}
                  />
                </CardContent>
              </Card>
            </TabsContent>

            {/* Advanced Tab */}
            <TabsContent value="advanced" className="space-y-4">
              <Card>
                <CardHeader>
                  <CardTitle>Advanced Settings</CardTitle>
                  <CardDescription>
                    Configure advanced simulator options
                  </CardDescription>
                </CardHeader>
                <CardContent className="space-y-4">
                  <FormField
                    control={form.control}
                    name="advanced.log_level"
                    render={({ field }) => (
                      <FormItem>
                        <FormLabel>Log Level</FormLabel>
                        <Select
                          onValueChange={field.onChange}
                          defaultValue={field.value}
                        >
                          <FormControl>
                            <SelectTrigger>
                              <SelectValue placeholder="Select log level" />
                            </SelectTrigger>
                          </FormControl>
                          <SelectContent>
                            <SelectItem value="error">Error</SelectItem>
                            <SelectItem value="warn">Warning</SelectItem>
                            <SelectItem value="info">Info</SelectItem>
                            <SelectItem value="debug">Debug</SelectItem>
                            <SelectItem value="trace">Trace</SelectItem>
                          </SelectContent>
                        </Select>
                        <FormDescription>
                          Logging verbosity level
                        </FormDescription>
                        <FormMessage />
                      </FormItem>
                    )}
                  />

                  <FormField
                    control={form.control}
                    name="advanced.max_log_entries"
                    render={({ field }) => (
                      <FormItem>
                        <FormLabel>Max Log Entries</FormLabel>
                        <FormControl>
                          <Input
                            type="number"
                            placeholder="10000"
                            {...field}
                            onChange={(e) => field.onChange(parseInt(e.target.value))}
                          />
                        </FormControl>
                        <FormDescription>
                          Maximum number of log entries to retain in memory
                        </FormDescription>
                        <FormMessage />
                      </FormItem>
                    )}
                  />

                  <FormField
                    control={form.control}
                    name="advanced.jwt_secret"
                    render={({ field }) => (
                      <FormItem>
                        <FormLabel>JWT Secret</FormLabel>
                        <div className="flex gap-2">
                          <FormControl>
                            <Input
                              type={revealedFields.has('jwt_secret') ? 'text' : 'password'}
                              placeholder="your-secret-key"
                              {...field}
                            />
                          </FormControl>
                          <Button
                            type="button"
                            variant="outline"
                            size="icon"
                            onClick={() => toggleFieldVisibility('jwt_secret')}
                          >
                            {revealedFields.has('jwt_secret') ? (
                              <EyeOff className="h-4 w-4" />
                            ) : (
                              <Eye className="h-4 w-4" />
                            )}
                          </Button>
                        </div>
                        <FormDescription>
                          Secret key for JWT token signing (leave empty to auto-generate)
                        </FormDescription>
                        <FormMessage />
                      </FormItem>
                    )}
                  />

                  <FormField
                    control={form.control}
                    name="advanced.jwt_expiry_hours"
                    render={({ field }) => (
                      <FormItem>
                        <FormLabel>JWT Expiry (hours)</FormLabel>
                        <FormControl>
                          <Input
                            type="number"
                            placeholder="24"
                            {...field}
                            onChange={(e) => field.onChange(parseInt(e.target.value))}
                          />
                        </FormControl>
                        <FormDescription>
                          JWT token expiration time in hours
                        </FormDescription>
                        <FormMessage />
                      </FormItem>
                    )}
                  />

                  <FormField
                    control={form.control}
                    name="advanced.allowed_origins"
                    render={({ field }) => (
                      <FormItem>
                        <FormLabel>Allowed Origins</FormLabel>
                        <FormControl>
                          <Input placeholder="*" {...field} />
                        </FormControl>
                        <FormDescription>
                          CORS allowed origins (comma-separated or * for all)
                        </FormDescription>
                        <FormMessage />
                      </FormItem>
                    )}
                  />

                  <FormField
                    control={form.control}
                    name="advanced.enable_websocket"
                    render={({ field }) => (
                      <FormItem className="flex flex-row items-center justify-between rounded-lg border p-4">
                        <div className="space-y-0.5">
                          <FormLabel className="text-base">
                            Enable WebSocket
                          </FormLabel>
                          <FormDescription>
                            Enable WebSocket for real-time updates
                          </FormDescription>
                        </div>
                        <FormControl>
                          <Switch
                            checked={field.value}
                            onCheckedChange={field.onChange}
                          />
                        </FormControl>
                      </FormItem>
                    )}
                  />
                </CardContent>
              </Card>
            </TabsContent>
          </Tabs>

          <div className="flex justify-end gap-4">
            <Button
              type="button"
              variant="outline"
              onClick={handleReset}
              disabled={saving}
            >
              <RotateCcw className="mr-2 h-4 w-4" />
              Reset
            </Button>
            <Button type="submit" disabled={saving}>
              {saving ? (
                <>
                  <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                  Saving...
                </>
              ) : (
                <>
                  <Save className="mr-2 h-4 w-4" />
                  Save Configuration
                </>
              )}
            </Button>
          </div>
        </form>
      </Form>
    </div>
  );
}
