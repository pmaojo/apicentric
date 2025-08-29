#!/bin/bash

# Script para probar el contract testing con Star Wars API
# Este script demuestra cómo usar Pulse para validar contratos API

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PULSE_DIR="$SCRIPT_DIR"
EXAMPLES_DIR="$PULSE_DIR/examples"

echo "🚀 Testing Pulse Contract Validation with Star Wars API"
echo "=============================================="

# Cambiar al directorio de Pulse
cd "$PULSE_DIR"

# Construir el proyecto
echo "📦 Building Pulse..."
cargo build --release

echo ""
echo "📋 Registering Star Wars API contract..."

# Registrar el contrato
cargo run --release -- contract register \
    --service "star-wars-api" \
    --spec "$EXAMPLES_DIR/swapi-service.yaml" \
    --description "Star Wars API contract for testing"

echo ""
echo "📝 Listing registered contracts..."
cargo run --release -- contract list

echo ""
echo "🔍 Getting contract details..."
# Obtener el ID del contrato (asumimos que es el primero)
CONTRACT_ID=$(cargo run --release -- contract list --format json 2>/dev/null | grep -o '"id":"[^"]*"' | head -1 | cut -d'"' -f4)

if [ -n "$CONTRACT_ID" ]; then
    echo "Contract ID found: $CONTRACT_ID"
    cargo run --release -- contract get "$CONTRACT_ID"
    
    echo ""
    echo "🧪 Running contract validation against real Star Wars API..."
    echo "This will:"
    echo "  1. Start a mock server based on the YAML spec"
    echo "  2. Send requests to both mock and real APIs"
    echo "  3. Compare responses for compatibility"
    echo ""
    
    # Ejecutar validación del contrato
    cargo run --release -- contract validate \
        --contract-id "$CONTRACT_ID" \
        --real-api-url "https://swapi.dev/api" \
        --timeout 30 \
        --retries 3 \
        --policy strict
else
    echo "❌ Could not find contract ID. Contract registration may have failed."
    exit 1
fi

echo ""
echo "✅ Contract testing completed!"
echo "Check the output above to see if the real Star Wars API matches our contract specification."
