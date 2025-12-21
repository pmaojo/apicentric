import { useState, useEffect } from 'react';
import { AlertCircle, RefreshCw, Settings, Terminal } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle } from '@/components/ui/card';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert';
import { setApiUrl, getApiUrl, resetApiUrl } from '@/services/api';

interface BackendConnectionErrorProps {
  error: Error;
  onRetry: () => void;
}

export function BackendConnectionError({ error, onRetry }: BackendConnectionErrorProps) {
  const [customUrl, setCustomUrl] = useState('');
  const [isUpdating, setIsUpdating] = useState(false);
  const currentUrl = getApiUrl();

  useEffect(() => {
    setCustomUrl(currentUrl);
  }, [currentUrl]);

  const handleUpdateUrl = () => {
    setIsUpdating(true);
    // Simple validation
    let url = customUrl.trim();
    if (!url) return;

    // Ensure protocol
    if (!url.startsWith('http://') && !url.startsWith('https://')) {
      url = `http://${url}`;
    }

    // Remove trailing slash
    if (url.endsWith('/')) {
      url = url.slice(0, -1);
    }

    setApiUrl(url);
    setIsUpdating(false);
    onRetry();
  };

  const handleResetUrl = () => {
    resetApiUrl();
    setCustomUrl(getApiUrl());
    onRetry();
  };

  const isMixedContent = typeof window !== 'undefined' &&
    window.location.protocol === 'https:' &&
    currentUrl.startsWith('http:');

  return (
    <div className="flex flex-col items-center justify-center min-h-[80vh] p-4 bg-background">
      <Card className="w-full max-w-2xl shadow-lg border-destructive/20">
        <CardHeader className="bg-destructive/5 rounded-t-xl pb-6">
          <div className="flex items-center gap-3 text-destructive mb-2">
            <AlertCircle className="h-8 w-8" />
            <CardTitle className="text-2xl">Connection Failed</CardTitle>
          </div>
          <CardDescription className="text-base text-muted-foreground ml-11">
            Unable to communicate with the Apicentric backend service.
          </CardDescription>
        </CardHeader>

        <CardContent className="pt-6 space-y-6">
          {/* Detailed Error Message */}
          <Alert variant="destructive">
            <AlertCircle className="h-4 w-4" />
            <AlertTitle>Error Details</AlertTitle>
            <AlertDescription className="font-mono text-xs mt-1">
              {error.message}
            </AlertDescription>
          </Alert>

          {/* Diagnostic Info */}
          <div className="grid gap-4 md:grid-cols-2">
            <div className="space-y-2">
              <h3 className="font-semibold text-sm flex items-center gap-2">
                <Settings className="h-4 w-4 text-muted-foreground" />
                Current Configuration
              </h3>
              <div className="text-sm border rounded-md p-3 bg-muted/50">
                <div className="grid grid-cols-[80px_1fr] gap-1">
                  <span className="text-muted-foreground">API URL:</span>
                  <span className="font-mono">{currentUrl}</span>
                  <span className="text-muted-foreground">Frontend:</span>
                  <span className="font-mono">{typeof window !== 'undefined' ? window.location.origin : 'unknown'}</span>
                </div>
              </div>
            </div>

            <div className="space-y-2">
              <h3 className="font-semibold text-sm flex items-center gap-2">
                <Terminal className="h-4 w-4 text-muted-foreground" />
                Troubleshooting
              </h3>
              <ul className="text-sm list-disc list-inside space-y-1 text-muted-foreground">
                <li>Check if the backend is running</li>
                <li>Verify the API URL is correct</li>
                {isMixedContent && (
                  <li className="text-amber-600 font-medium">
                    Mixed Content detected (HTTPS → HTTP)
                  </li>
                )}
                <li>Check network console for CORS errors</li>
              </ul>
            </div>
          </div>

          {/* URL Configuration */}
          <div className="space-y-3 pt-2 border-t">
            <Label htmlFor="api-url">Backend API URL</Label>
            <div className="flex gap-2">
              <Input
                id="api-url"
                value={customUrl}
                onChange={(e) => setCustomUrl(e.target.value)}
                placeholder="http://localhost:8080"
                className="font-mono"
              />
              <Button onClick={handleUpdateUrl} disabled={isUpdating}>
                Update & Retry
              </Button>
            </div>
            <p className="text-xs text-muted-foreground">
              If running locally, try <code className="bg-muted px-1 rounded">http://localhost:8080</code>.
              If deployed, ensure the backend is accessible over HTTPS.
            </p>
          </div>
        </CardContent>

        <CardFooter className="bg-muted/10 flex justify-between rounded-b-xl">
          <Button variant="ghost" size="sm" onClick={handleResetUrl}>
            Reset to Default
          </Button>
          <Button variant="default" onClick={onRetry} className="gap-2">
            <RefreshCw className="h-4 w-4" />
            Try Again
          </Button>
        </CardFooter>
      </Card>
    </div>
  );
}
