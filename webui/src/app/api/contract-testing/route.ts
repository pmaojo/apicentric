
import { NextResponse } from 'next/server';
import type { NextRequest } from 'next/server';
import type { Service } from '@/lib/types';

/**
 * @fileoverview API route for running contract tests against a service.
 */

type TestResult = {
    endpoint: string;
    method: 'GET' | 'POST' | 'PUT' | 'DELETE';
    mockStatus: number;
    realStatus: number;
    compatible: boolean;
};

/**
 * Handles POST requests to run contract tests.
 * @param {NextRequest} req - The incoming Next.js request.
 * @returns {Promise<NextResponse>} A response object with the test results.
 */
export async function POST(req: NextRequest) {
  try {
    const service: Service = await req.json();

    if (!service || !service.endpoints) {
      return NextResponse.json({ error: 'Invalid service definition provided.' }, { status: 400 });
    }

    // Simulate running tests with a delay
    await new Promise(resolve => setTimeout(resolve, 1500));

    const results: TestResult[] = service.endpoints.map(endpoint => {
        const mockStatus = Math.random() > 0.1 ? 200 : 500;
        const realStatus = Math.random() > 0.2 ? 200 : 404;
        return {
            endpoint: endpoint.path,
            method: endpoint.method as any,
            mockStatus,
            realStatus,
            compatible: mockStatus === realStatus,
        };
    });

    return NextResponse.json(results);

  } catch (error) {
    console.error('Contract testing error:', error);
    const errorMessage = error instanceof Error ? error.message : 'An unknown error occurred.';
    return NextResponse.json({ error: `Failed to run contract tests: ${errorMessage}` }, { status: 500 });
  }
}
