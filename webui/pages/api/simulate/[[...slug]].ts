import type { NextApiRequest, NextApiResponse } from 'next';
import { spawn } from 'child_process';
import path from 'path';
import fs from 'fs';

// Configuration
const BACKEND_DIR = path.join(process.cwd(), 'webui/backend');
const BINARY_NAME = 'apicentric';
const BINARY_PATH = path.join(BACKEND_DIR, BINARY_NAME);
const SERVICES_DIR = path.join(process.cwd(), 'services'); // Assume services are at root or configure appropriately

export default async function handler(
  req: NextApiRequest,
  res: NextApiResponse
) {
  // 1. Determine the path and service
  // The route is /api/simulate/[[...slug]]
  // Example: /api/simulate/users/123 -> path: /users/123
  const { slug } = req.query;
  const requestPath = Array.isArray(slug) ? '/' + slug.join('/') : '/';

  // For this example, we'll assume a single default service file.
  // In a real scenario, you might map the first part of the path to a service file.
  const serviceFile = path.join(SERVICES_DIR, 'default.yaml');

  if (!fs.existsSync(BINARY_PATH)) {
    return res.status(500).json({ error: 'Apicentric binary not found', path: BINARY_PATH });
  }

  // 2. Spawn the process
  const args = [
    'simulator',
    'handle-request',
    '--service', serviceFile,
    '--method', req.method || 'GET',
    '--path', requestPath,
  ];

  if (req.body && (typeof req.body === 'object' || typeof req.body === 'string')) {
    const bodyStr = typeof req.body === 'string' ? req.body : JSON.stringify(req.body);
    args.push('--body', bodyStr);
  }

  const headers = JSON.stringify(req.headers);
  args.push('--headers', headers);

  const child = spawn(BINARY_PATH, args);

  let stdout = '';
  let stderr = '';

  child.stdout.on('data', (data) => {
    stdout += data.toString();
  });

  child.stderr.on('data', (data) => {
    stderr += data.toString();
  });

  const exitCode = await new Promise<number>((resolve) => {
    child.on('close', (code) => {
      resolve(code || 0);
    });
  });

  if (exitCode !== 0) {
    console.error('Apicentric error:', stderr);
    return res.status(500).json({ error: 'Simulator failed', details: stderr });
  }

  try {
    const result = JSON.parse(stdout);

    // Set response headers
    if (result.headers) {
      for (const [key, value] of Object.entries(result.headers)) {
        res.setHeader(key, value as string);
      }
    }

    // Set status
    res.status(result.status || 200);

    // Send body
    // If the body is a string that looks like JSON, Next.js will handle it if we send it as object?
    // The result.body is a string.
    res.send(result.body);

  } catch (e) {
    console.error('Failed to parse simulator output:', e, stdout);
    res.status(500).json({ error: 'Invalid simulator output', output: stdout });
  }
}
