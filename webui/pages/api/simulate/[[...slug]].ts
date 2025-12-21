import type { NextApiRequest, NextApiResponse } from 'next';
import { spawn } from 'child_process';
import path from 'path';
import fs from 'fs';

// Configuration
const BACKEND_DIR = path.join(process.cwd(), 'webui/backend');
const BINARY_NAME = 'apicentric';
const BINARY_PATH = path.join(BACKEND_DIR, BINARY_NAME);
const SERVICES_DIR = path.join(process.cwd(), 'services');

export default async function handler(
  req: NextApiRequest,
  res: NextApiResponse
) {
  const { slug } = req.query;
  const requestPath = Array.isArray(slug) ? '/' + slug.join('/') : '/';

  // In Vercel, we need to ensure this file exists.
  // We'll map to default.yaml for this example.
  const serviceFile = path.join(SERVICES_DIR, 'default.yaml');

  if (!fs.existsSync(BINARY_PATH)) {
    return res.status(500).json({ error: 'Apicentric binary not found', path: BINARY_PATH });
  }

  const args = [
    'simulator',
    'handle-request',
    '--service', serviceFile,
    '--method', req.method || 'GET',
    '--path', requestPath,
  ];

  const headers = JSON.stringify(req.headers);
  args.push('--headers', headers);

  // Body handling: We pass body via stdin
  let bodyContent = '';
  if (req.body) {
    if (typeof req.body === 'string') {
        bodyContent = req.body;
    } else if (typeof req.body === 'object') {
        bodyContent = JSON.stringify(req.body);
    }
  }

  const child = spawn(BINARY_PATH, args);

  // Write body to stdin
  if (bodyContent) {
      child.stdin.write(bodyContent);
  }
  child.stdin.end();

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

    if (result.headers) {
      for (const [key, value] of Object.entries(result.headers)) {
        res.setHeader(key, value as string);
      }
    }

    res.status(result.status || 200);
    res.send(result.body);

  } catch (e) {
    console.error('Failed to parse simulator output:', e, stdout);
    res.status(500).json({ error: 'Invalid simulator output', output: stdout });
  }
}
