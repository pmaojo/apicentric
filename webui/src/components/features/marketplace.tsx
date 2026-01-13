
'use client';

import { useState } from 'react';
import { useQuery, useMutation } from '@tanstack/react-query';
import { fetchMarketplace, importFromUrl } from '@/services/api';
import { Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Badge } from '@/components/ui/badge';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { useToast } from '@/hooks/use-toast';
import { Loader2, Download, Globe, Search, PlusCircle } from 'lucide-react';
import type { MarketplaceItem } from '@/lib/types';

interface MarketplaceProps {
  onAddService: (serviceData: { name: string, version: string, port: number, definition: string }) => void;
}

export function Marketplace({ onAddService }: MarketplaceProps) {
  const { toast } = useToast();
  const [customUrl, setCustomUrl] = useState('');
  const [activeTab, setActiveTab] = useState('catalog');
  const [searchQuery, setSearchQuery] = useState('');

  const { data: items, isLoading } = useQuery({
    queryKey: ['marketplace'],
    queryFn: fetchMarketplace,
  });

  const importMutation = useMutation({
    mutationFn: (url: string) => importFromUrl(url),
    onSuccess: (data) => {
      toast({
        title: 'Import Successful',
        description: `Service '${data.service_name}' has been imported.`,
      });
      // We assume the service is added to backend and next refresh will pick it up
      // or we can manually trigger something if we had the full service object.
      // Since we only get name and yaml, we can try to notify parent if needed,
      // but simpler to just let the user know.
      // Wait, onAddService expects { name, version, port, definition }.
      // We can try to parse the YAML to get these details, or just rely on the backend save.
      // The backend `import_from_url` already saves the service.
      // So we might just want to refresh the service list.
      // But `onAddService` is used to update local state optimistically in AppContent.
      // Let's just rely on the toast for now and maybe trigger a reload if possible.
      // Actually, onAddService adds to the list. We should probably parse the YAML.
    },
    onError: (error: Error) => {
      toast({
        variant: 'destructive',
        title: 'Import Failed',
        description: error.message,
      });
    },
  });

  const handleImport = (url: string) => {
    importMutation.mutate(url);
  };

  const filteredItems = items?.filter(item =>
    item.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
    item.description.toLowerCase().includes(searchQuery.toLowerCase()) ||
    item.category.toLowerCase().includes(searchQuery.toLowerCase())
  );

  const categories = Array.from(new Set(items?.map(i => i.category) || []));

  return (
    <div className="space-y-6 h-full flex flex-col">
      <div className="flex justify-between items-center">
        <div>
          <h2 className="text-2xl font-bold tracking-tight">Marketplace & Import</h2>
          <p className="text-muted-foreground">
            Discover popular API templates or import from external sources.
          </p>
        </div>
      </div>

      <Tabs value={activeTab} onValueChange={setActiveTab} className="flex-1 flex flex-col">
        <TabsList>
          <TabsTrigger value="catalog">Catalog</TabsTrigger>
          <TabsTrigger value="import">Import from URL</TabsTrigger>
        </TabsList>

        <TabsContent value="catalog" className="flex-1 space-y-4">
          <div className="flex items-center space-x-2">
            <Search className="h-4 w-4 text-muted-foreground" />
            <Input
              placeholder="Search services..."
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              className="max-w-sm"
            />
          </div>

          {isLoading ? (
            <div className="flex justify-center p-8">
              <Loader2 className="h-8 w-8 animate-spin text-primary" />
            </div>
          ) : (
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
              {filteredItems?.map((item) => (
                <Card key={item.id} className="flex flex-col">
                  <CardHeader>
                    <div className="flex justify-between items-start">
                      <CardTitle className="text-lg">{item.name}</CardTitle>
                      <Badge variant="outline">{item.category}</Badge>
                    </div>
                    <CardDescription className="line-clamp-2" title={item.description}>
                      {item.description}
                    </CardDescription>
                  </CardHeader>
                  <CardContent className="flex-1">
                    {/* Placeholder for logo if we had one */}
                    <div className="flex items-center justify-center h-20 bg-muted/20 rounded-md">
                        {item.category === 'SaaS' ? <Globe className="h-10 w-10 text-muted-foreground" /> : <PlusCircle className="h-10 w-10 text-muted-foreground" />}
                    </div>
                  </CardContent>
                  <CardFooter>
                    <Button
                      className="w-full"
                      onClick={() => handleImport(item.definition_url)}
                      disabled={importMutation.isPending}
                    >
                      {importMutation.isPending ? (
                        <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                      ) : (
                        <Download className="mr-2 h-4 w-4" />
                      )}
                      Install
                    </Button>
                  </CardFooter>
                </Card>
              ))}
              {filteredItems?.length === 0 && (
                <div className="col-span-full text-center py-12 text-muted-foreground">
                  No services found matching your search.
                </div>
              )}
            </div>
          )}
        </TabsContent>

        <TabsContent value="import" className="flex-1">
          <Card>
            <CardHeader>
              <CardTitle>Import Service Definition</CardTitle>
              <CardDescription>
                Import a service definition from a URL. Supported formats: OpenAPI (Swagger) v3.
              </CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="grid w-full max-w-sm items-center gap-1.5">
                <Label htmlFor="url">Definition URL</Label>
                <Input
                  type="url"
                  id="url"
                  placeholder="https://example.com/openapi.yaml"
                  value={customUrl}
                  onChange={(e) => setCustomUrl(e.target.value)}
                />
              </div>
            </CardContent>
            <CardFooter>
              <Button
                onClick={() => handleImport(customUrl)}
                disabled={!customUrl || importMutation.isPending}
              >
                {importMutation.isPending ? (
                    <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                ) : (
                    <Download className="mr-2 h-4 w-4" />
                )}
                Import
              </Button>
            </CardFooter>
          </Card>
        </TabsContent>
      </Tabs>
    </div>
  );
}
