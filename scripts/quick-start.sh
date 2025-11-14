#!/bin/bash
# Quick start script for Apicentric development

set -e

echo "🚀 Starting Apicentric..."

# Check if .env exists
if [ ! -f .env ]; then
    echo "📝 Creating .env from template..."
    cp .env.example .env
    echo "⚠️  Please edit .env with your configuration"
fi

# Check if Docker is available
if command -v docker &> /dev/null; then
    echo "🐳 Starting with Docker Compose..."
    docker-compose --profile dev up
else
    echo "📦 Docker not found, starting manually..."
    
    # Start backend in background
    echo "🦀 Starting backend..."
    cargo run --features gui,cloud -- cloud --port 8000 &
    BACKEND_PID=$!
    
    # Start frontend
    echo "⚛️  Starting frontend..."
    cd webui
    npm install
    npm run dev &
    FRONTEND_PID=$!
    cd ..
    
    # Trap to kill processes on exit
    trap "kill $BACKEND_PID $FRONTEND_PID" EXIT
    
    echo ""
    echo "✅ Apicentric is running!"
    echo "   Frontend: http://localhost:3000"
    echo "   Backend:  http://localhost:8000"
    echo ""
    echo "Press Ctrl+C to stop"
    
    wait
fi
