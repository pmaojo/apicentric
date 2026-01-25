'use client';

import * as React from 'react';
import { useSearchParams } from 'next/navigation';
import { getService } from '@/services/api';
import { Service } from '@/lib/types';
import { Loader2 } from 'lucide-react';
import { ServiceEditorLayout } from '@/components/features/service-editor/layout';

export default function ServiceEditorPage() {
  return (
    <React.Suspense fallback={<div className="flex h-screen items-center justify-center"><Loader2 className="h-8 w-8 animate-spin text-primary" /></div>}>
      <ServiceEditorContent />
    </React.Suspense>
  );
}

function ServiceEditorContent() {
  const searchParams = useSearchParams();
  const serviceName = searchParams.get('service');
  const [service, setService] = React.useState<Service | null>(null);
  const [loading, setLoading] = React.useState(true);
  const [error, setError] = React.useState<string | null>(null);

  React.useEffect(() => {
    if (!serviceName) {
      setLoading(false);
      return;
    }

    const loadService = async () => {
      try {
        const data = await getService(serviceName);
        // Transform ServiceResponse to Service if needed, or stick to a unified type.
        // For now, assuming data has the shape we need or we map it.
        // The API returns { info: ApiService, yaml: string }
        // We'll need to parse the YAML or use the info to populate the editor state.
        
        // Let's create a partial service object from the response
        // In a real app we might want a transformer.
        // For the editor, we primarily need the name and the definition (yaml).
        
        const serviceData: Service = {
            id: data.info.id || 'unknown',
            name: data.info.name,
            version: data.info.version || '1.0.0',
            port: data.info.port || 3000,
            status: data.info.is_running ? 'running' : 'stopped',
            endpoints: [], // We will parse from YAML
            definition: data.yaml
        };

        setService(serviceData);
      } catch (err) {
        setError(err instanceof Error ? err.message : 'Failed to load service');
      } finally {
        setLoading(false);
      }
    };

    loadService();
  }, [serviceName]);

  if (loading) {
    return (
      <div className="flex h-screen items-center justify-center">
        <Loader2 className="h-8 w-8 animate-spin text-primary" />
      </div>
    );
  }

  if (error || !service) {
    return (
      <div className="flex h-screen items-center justify-center flex-col gap-4">
        <h1 className="text-xl font-semibold text-destructive">Error Loading Service</h1>
        <p className="text-muted-foreground">{error || 'Service not specified or not found.'}</p>
      </div>
    );
  }

  return <ServiceEditorLayout service={service} />;
}
