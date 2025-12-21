import { useState, useEffect } from 'react';
import { AlertCircle, RefreshCw, Settings, Terminal, ExternalLink } from 'lucide-react';
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
  const isVercel = typeof window !== 'undefined' && window.location.hostname.includes('vercel.app');

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

          {isVercel && (
             <Alert className="bg-amber-50 border-amber-200 text-amber-800">
                <ExternalLink className="h-4 w-4 text-amber-600" />
                <AlertTitle className="text-amber-800">Vercel Deployment Detected</AlertTitle>
                <AlertDescription className="text-amber-700 text-sm mt-1">
                   The backend binary cannot run directly on Vercel's serverless environment.
                   You must deploy the backend separately (e.g., on Fly.io, Railway, or a VPS) and set the
                   <code className="bg-amber-100 px-1 mx-1 rounded text-amber-900 font-mono">BACKEND_URL</code>
                   environment variable in your Vercel project settings.
                </AlertDescription>
             </Alert>
          )}

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
                  <span className="font-mono break-all">{currentUrl}</span>
                  <span className="text-muted-foreground">Frontend:</span>
                  <span className="font-mono break-all">{typeof window !== 'undefined' ? window.location.origin : 'unknown'}</span>
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
            <Label htmlFor="api-url">Backend API URL Override</Label>
            <div className="flex gap-2">
              <Input
                id="api-url"
                value={customUrl}
                onChange={(e) => setCustomUrl(e.target.value)}
                placeholder="http://localhost:8080 or https://api.myapp.com"
                className="font-mono"
              />
              <Button onClick={handleUpdateUrl} disabled={isUpdating}>
                Update & Retry
              </Button>
            </div>
            <p className="text-xs text-muted-foreground">
              This overrides the default connection URL for your session.
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
