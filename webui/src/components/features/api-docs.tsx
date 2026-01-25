import React, { useState, useEffect } from 'react';
import SwaggerUI from 'swagger-ui-react';
import 'swagger-ui-react/swagger-ui.css';
import { fetchServiceOpenApi } from '@/services/api';
import { Loader2, AlertCircle } from 'lucide-react';
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert';

interface ApiDocsProps {
  serviceName: string;
}

export function ApiDocs({ serviceName }: ApiDocsProps) {
  const [spec, setSpec] = useState<object | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let mounted = true;

    async function loadSpec() {
      try {
        setLoading(true);
        setError(null);
        const data = await fetchServiceOpenApi(serviceName);
        if (mounted) {
          setSpec(data);
        }
      } catch (err) {
        if (mounted) {
          console.error('Failed to load OpenAPI spec:', err);
          setError(err instanceof Error ? err.message : 'Failed to load API documentation');
        }
      } finally {
        if (mounted) {
          setLoading(false);
        }
      }
    }

    if (serviceName) {
      loadSpec();
    }

    return () => {
      mounted = false;
    };
  }, [serviceName]);

  if (loading) {
    return (
      <div className="flex h-64 items-center justify-center">
        <Loader2 className="h-8 w-8 animate-spin text-primary" />
        <span className="ml-2 text-sm text-muted-foreground">Generating API documentation...</span>
      </div>
    );
  }

  if (error) {
    return (
      <Alert variant="destructive">
        <AlertCircle className="h-4 w-4" />
        <AlertTitle>Error</AlertTitle>
        <AlertDescription>{error}</AlertDescription>
      </Alert>
    );
  }

  return (
    <div className="api-docs-container h-[600px] overflow-y-auto rounded-md border bg-white">
      {spec && (
        <SwaggerUI 
          spec={spec} 
          docExpansion="list"
          defaultModelsExpandDepth={-1} // Hide models by default to save space
        />
      )}
    </div>
  );
}
