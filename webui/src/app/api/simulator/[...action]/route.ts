import { NextResponse } from 'next/server';
import type { NextRequest } from 'next/server';
import yaml from 'js-yaml';

async function handleSimulatorAction(action: string[], req: NextRequest) {
    const actionPath = action.join('/');
    
    let body;
    try {
        body = await req.json();
    } catch (e) {
        body = null;
    }

    switch (actionPath) {
        case 'start':
            // Placeholder: In a real implementation, this would call the Rust backend
            console.log('API call to start simulator:', body);
            return NextResponse.json({ success: true, message: 'Simulator start initiated.' });

        case 'stop':
            // Placeholder
            console.log('API call to stop simulator:', body);
            return NextResponse.json({ success: true, message: 'Simulator stop initiated.' });
            
        case 'status':
             // Placeholder
            console.log('API call to get simulator status');
            return NextResponse.json({ success: true, data: { status: "running", services: [] } });
            
        case 'validate':
            console.log('API call to validate service definition');
            if (!body || !body.definition) {
                return NextResponse.json({ success: false, error: 'No definition provided.' }, { status: 400 });
            }
            try {
                yaml.load(body.definition);
                return NextResponse.json({ success: true, message: 'YAML is valid.' });
            } catch (e) {
                const error = e instanceof Error ? e.message : 'An unknown error occurred.';
                return NextResponse.json({ success: false, error: `Invalid YAML: ${error}` }, { status: 400 });
            }

        default:
            return NextResponse.json({ error: 'Unknown simulator action' }, { status: 404 });
    }
}


export async function POST(
  req: NextRequest,
  { params }: { params: { action: string[] } }
) {
  return handleSimulatorAction(params.action, req);
}

export async function GET(
  req: NextRequest,
  { params }: { params: { action: string[] } }
) {
  return handleSimulatorAction(params.action, req);
}
