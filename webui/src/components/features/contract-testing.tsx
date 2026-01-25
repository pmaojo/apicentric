
'use client';

import * as React from 'react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { FileCheck, PlayCircle, HardDrive, TestTube, Cloud, AlertTriangle, CheckCircle2, XCircle, Loader2 } from 'lucide-react';
import type { ApiService, Service } from '@/lib/types';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '../ui/select';
import { Skeleton } from '../ui/skeleton';
import { Progress } from '../ui/progress';
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '../ui/table';
import { Badge } from '../ui/badge';
import { useToast } from '@/hooks/use-toast';
import { useMutation } from '@tanstack/react-query';
import { runContractTests } from '@/services/api';

/**
 * @fileoverview A component for running and displaying contract testing results.
 */

type TestResult = {
    endpoint: string;
    method: 'GET' | 'POST' | 'PUT' | 'DELETE';
    mockStatus: number;
    realStatus: number;
    compatible: boolean;
};

type ContractTestingProps = {
    services: Service[];
}

/**
 * Renders the Contract Testing UI, allowing users to select a service,
 * run validation tests, and view the results.
 * @param {ContractTestingProps} props - The component props.
 * @returns {React.ReactElement} The rendered ContractTesting component.
 */
export function ContractTesting({ services }: ContractTestingProps) {
    const [selectedService, setSelectedService] = React.useState<Service | null>(null);
    const [selectedEnvironment, setSelectedEnvironment] = React.useState('staging');
    const { toast } = useToast();

    const { mutate: runTests, data: testResults, isPending: isRunningTests, error } = useMutation<TestResult[], Error, Service>({
        mutationFn: runContractTests,
        onSuccess: () => {
            toast({
                title: "Validation Complete",
                description: "Contract tests have finished running.",
            });
        },
        onError: (err) => {
            toast({
                variant: "destructive",
                title: "Validation Failed",
                description: err.message,
            });
        },
    });

    const handleRunValidation = () => {
        if (!selectedService) return;
        runTests(selectedService);
    };

    const overallCompatibility = React.useMemo(() => {
        if (!testResults || !Array.isArray(testResults) || testResults.length === 0) return null;
        const compatibleCount = testResults.filter(r => r.compatible).length;
        return (compatibleCount / testResults.length) * 100;
    }, [testResults]);

    const getStatusColor = (status: number) => {
        if (status >= 500) return 'bg-red-500/20 text-red-400 border-red-500/30';
        if (status >= 400) return 'bg-yellow-500/20 text-yellow-400 border-yellow-500/30';
        if (status >= 200) return 'bg-green-500/20 text-green-400 border-green-500/30';
        return 'bg-gray-500/20 text-gray-400 border-gray-500/30';
    };

    return (
        <Card>
            <CardHeader>
                <CardTitle>Contract Testing</CardTitle>
                <CardDescription>
                    Validate your mock services against real API implementations to ensure compatibility.
                </CardDescription>
            </CardHeader>
            <CardContent className="space-y-6">
                <div className="grid gap-4 sm:grid-cols-2">
                    <div className="space-y-2">
                        <label className="text-sm font-medium">Select Service</label>
                        {services.length === 0 ? (
                            <Skeleton className="h-10 w-full" />
                        ) : (
                            <Select onValueChange={(value) => setSelectedService(services.find(s => s.name === value) || null)} value={selectedService?.name || ""}>
                                <SelectTrigger>
                                    <SelectValue placeholder="Choose a service to test..." />
                                </SelectTrigger>
                                <SelectContent>
                                    {services.map((service) => (
                                        <SelectItem key={service.id} value={service.name}>
                                            {service.name}
                                        </SelectItem>
                                    ))}
                                </SelectContent>
                            </Select>
                        )}
                    </div>
                    <div className="space-y-2">
                        <label className="text-sm font-medium">Target Environment</label>
                        <Select onValueChange={setSelectedEnvironment} defaultValue="staging">
                            <SelectTrigger>
                                <SelectValue placeholder="Select environment" />
                            </SelectTrigger>
                            <SelectContent>
                                <SelectItem value="staging"><div className="flex items-center"><TestTube className="mr-2 h-4 w-4" />Staging</div></SelectItem>
                                <SelectItem value="production"><div className="flex items-center"><Cloud className="mr-2 h-4 w-4" />Production</div></SelectItem>
                                <SelectItem value="local"><div className="flex items-center"><HardDrive className="mr-2 h-4 w-4" />Local</div></SelectItem>
                            </SelectContent>
                        </Select>
                    </div>
                </div>

                <div className="flex justify-end">
                    <Button onClick={handleRunValidation} disabled={!selectedService || isRunningTests}>
                        {isRunningTests ? <Loader2 className="mr-2 h-4 w-4 animate-spin" /> : <PlayCircle className="mr-2 h-4 w-4" />}
                        {isRunningTests ? 'Running Validation...' : 'Run Validation'}
                    </Button>
                </div>

                <div className="space-y-4">
                    <h3 className="text-lg font-semibold">Results</h3>
                    {!testResults && !isRunningTests && !error && (
                        <div className="rounded-md border min-h-[200px] flex items-center justify-center bg-muted/50">
                            <div className="text-center text-muted-foreground">
                                <FileCheck className="mx-auto h-12 w-12" />
                                <p className="mt-2">Validation results will be shown here.</p>
                            </div>
                        </div>
                    )}
                    {isRunningTests && (
                        <div className="rounded-md border min-h-[200px] flex flex-col items-center justify-center bg-muted/50 p-8">
                             <Loader2 className="h-10 w-10 animate-spin text-primary mb-4" />
                            <p className="mb-4">Testing endpoints for &quot;{selectedService?.name}&quot;...</p>
                        </div>
                    )}
                    {error && (
                        <div className="rounded-md border min-h-[200px] flex items-center justify-center bg-destructive/10 text-destructive-foreground">
                             <AlertTriangle className="h-10 w-10 mr-4" />
                             <div>
                                <p className="font-bold">An error occurred</p>
                                <p className="text-sm">{error.message}</p>
                             </div>
                        </div>
                    )}
                    {testResults && (
                        <div>
                             <div className="mb-4 flex items-center gap-4">
                                {overallCompatibility === 100 ? (
                                    <CheckCircle2 className="h-10 w-10 text-green-500" />
                                ) : (
                                    <AlertTriangle className="h-10 w-10 text-yellow-500" />
                                )}
                                <div>
                                    <p className="text-xl font-bold">{overallCompatibility?.toFixed(0)}% Compatible</p>
                                    <p className="text-muted-foreground">
                                        {Array.isArray(testResults) ? testResults.filter(r => r.compatible).length : 0} of {Array.isArray(testResults) ? testResults.length : 0} endpoints are compatible.
                                    </p>
                                </div>
                            </div>
                            <div className="rounded-md border">
                                <Table>
                                    <TableHeader>
                                        <TableRow>
                                            <TableHead>Endpoint</TableHead>
                                            <TableHead>Mock Status</TableHead>
                                            <TableHead>Real Status</TableHead>
                                            <TableHead className="text-right">Result</TableHead>
                                        </TableRow>
                                    </TableHeader>
                                    <TableBody>
                                        {Array.isArray(testResults) ? testResults.map(result => (
                                            <TableRow key={result.endpoint + result.method}>
                                                <TableCell className="font-mono">{result.method} {result.endpoint}</TableCell>
                                                <TableCell><Badge variant="outline" className={getStatusColor(result.mockStatus)}>{result.mockStatus}</Badge></TableCell>
                                                <TableCell><Badge variant="outline" className={getStatusColor(result.realStatus)}>{result.realStatus}</Badge></TableCell>
                                                <TableCell className="text-right">
                                                    {result.compatible ? <CheckCircle2 className="h-5 w-5 text-green-500 ml-auto" /> : <XCircle className="h-5 w-5 text-red-500 ml-auto" />}
                                                </TableCell>
                                            </TableRow>
                                        )) : (
                                            <TableRow>
                                                <TableCell colSpan={4} className="text-center py-4 text-muted-foreground">
                                                    No results found
                                                </TableCell>
                                            </TableRow>
                                        )}
                                    </TableBody>
                                </Table>
                            </div>
                        </div>
                    )}
                </div>
            </CardContent>
        </Card>
    );
}
