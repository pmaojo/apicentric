#!/bin/bash

set -euo pipefail

# Configuración
INTEGRATION_BRANCH="integration/api-wip"
SOURCE_BRANCH="origin/API_WIP"
LOG_DIR=".kiro/specs/api-wip-integration/logs"
LOG_FILE="$LOG_DIR/integration-log-$(date +%Y%m%d_%H%M%S).txt"

# Definición de batches
declare -A BATCHES=(
    ["1"]="config:.gitignore,Cargo.toml"
    ["2"]="tests:tests/auth.rs,tests/cloud_api.rs"
    ["3"]="api-cmd:src/commands/api.rs"
    ["4"]="cloud:src/cloud/handlers.rs,src/cloud/server.rs"
    ["5"]="cli:src/bin/apicentric.rs,src/cli/mod.rs"
    ["6"]="commands:src/commands/mod.rs,src/commands/ai.rs,src/commands/simulator/export.rs"
    ["7"]="final:src/lib.rs,tests/cli_commands.rs,tests/cli_output.rs,tests/dockerize.rs,tests/gui/log_tests.rs"
)

# Funciones de logging
log() { 
    echo "[$(date +'%Y-%m-%d %H:%M:%S')] [INFO] $*" | tee -a "$LOG_FILE"
}

error() { 
    echo "[$(date +'%Y-%m-%d %H:%M:%S')] [ERROR] $*" | tee -a "$LOG_FILE"
}

success() { 
    echo "[$(date +'%Y-%m-%d %H:%M:%S')] [SUCCESS] $*" | tee -a "$LOG_FILE"
}

warn() { 
    echo "[$(date +'%Y-%m-%d %H:%M:%S')] [WARN] $*" | tee -a "$LOG_FILE"
}

# Función para configurar la rama de integración
setup_integration_branch() {
    log "Setting up integration branch..."
    
    # Fetch de la rama API_WIP
    log "Fetching $SOURCE_BRANCH..."
    if ! git fetch origin API_WIP 2>&1 | tee -a "$LOG_FILE"; then
        error "Failed to fetch $SOURCE_BRANCH"
        return 1
    fi
    
    # Verificar si la rama de integración ya existe
    if git show-ref --verify --quiet refs/heads/$INTEGRATION_BRANCH; then
        log "Integration branch already exists, checking out..."
        git checkout $INTEGRATION_BRANCH
    else
        log "Creating new integration branch from master..."
        git checkout -b $INTEGRATION_BRANCH master
    fi
    
    success "Integration branch ready: $INTEGRATION_BRANCH"
}

# Función para aplicar un batch
apply_batch() {
    local batch_id=$1
    local batch_info=${BATCHES[$batch_id]}
    local batch_name=$(echo "$batch_info" | cut -d: -f1)
    local files=$(echo "$batch_info" | cut -d: -f2)
    
    log "Applying Batch $batch_id: $batch_name"
    log "Files to integrate: $files"
    
    IFS=',' read -ra FILE_ARRAY <<< "$files"
    for file in "${FILE_ARRAY[@]}"; do
        log "  Checking out: $file"
        if git checkout $SOURCE_BRANCH -- "$file" 2>&1 | tee -a "$LOG_FILE"; then
            success "  Successfully checked out: $file"
        else
            error "Failed to checkout $file"
            return 1
        fi
    done
    
    # Commit los cambios
    git add .
    if git commit -m "Batch $batch_id: Integrate $batch_name from API_WIP" 2>&1 | tee -a "$LOG_FILE"; then
        success "Committed Batch $batch_id"
    else
        warn "No changes to commit for batch $batch_id (possibly already applied)"
    fi
    
    return 0
}

# Función para ejecutar validación
run_validation() {
    local batch_id=$1
    
    log "Running validation for Batch $batch_id..."
    log "=========================================="
    
    # Step 1: Compilación
    log "Step 1/4: Building project..."
    if cargo build --release 2>&1 | tee -a "$LOG_FILE"; then
        success "Build passed"
    else
        error "Build failed for Batch $batch_id"
        return 1
    fi
    
    # Step 2: Tests
    log "Step 2/4: Running tests..."
    if cargo test --all 2>&1 | tee -a "$LOG_FILE"; then
        success "Tests passed"
    else
        error "Tests failed for Batch $batch_id"
        return 1
    fi
    
    # Step 3: Clippy
    log "Step 3/4: Running clippy..."
    if cargo clippy -- -D warnings 2>&1 | tee -a "$LOG_FILE"; then
        success "Clippy passed"
    else
        warn "Clippy warnings detected (non-blocking)"
    fi
    
    # Step 4: Format check
    log "Step 4/4: Checking format..."
    if cargo fmt -- --check 2>&1 | tee -a "$LOG_FILE"; then
        success "Format check passed"
    else
        warn "Format issues detected (non-blocking)"
    fi
    
    log "=========================================="
    success "Validation completed for Batch $batch_id"
    return 0
}

# Función para crear punto de rollback
create_rollback_point() {
    local batch_id=$1
    local tag="batch-$batch_id-complete"
    
    log "Creating rollback point: $tag"
    git tag -f "$tag"
    success "Rollback point created: $tag"
    log "To rollback to this point: git reset --hard $tag"
}

# Función para integrar un batch completo
integrate_batch() {
    local batch_id=$1
    
    log ""
    log "=========================================="
    log "Starting Batch $batch_id"
    log "=========================================="
    
    if ! apply_batch "$batch_id"; then
        error "Failed to apply Batch $batch_id"
        return 1
    fi
    
    if ! run_validation "$batch_id"; then
        error "Validation failed for Batch $batch_id"
        if [ "$batch_id" -gt 1 ]; then
            log "To revert: git reset --hard batch-$((batch_id-1))-complete"
        else
            log "To revert: git reset --hard master"
        fi
        return 1
    fi
    
    create_rollback_point "$batch_id"
    success "Batch $batch_id completed successfully"
    log ""
    
    return 0
}

# Función principal
main() {
    local start_batch=${1:-1}
    local end_batch=${2:-7}
    
    # Crear directorio de logs si no existe
    mkdir -p "$LOG_DIR"
    
    log "=========================================="
    log "API_WIP Integration Process Started"
    log "=========================================="
    log "Start Batch: $start_batch"
    log "End Batch: $end_batch"
    log "Log File: $LOG_FILE"
    log "Integration Branch: $INTEGRATION_BRANCH"
    log "Source Branch: $SOURCE_BRANCH"
    log ""
    
    # Setup de la rama de integración
    if ! setup_integration_branch; then
        error "Failed to setup integration branch"
        exit 1
    fi
    
    # Integrar batches
    for batch_id in $(seq $start_batch $end_batch); do
        if ! integrate_batch "$batch_id"; then
            error "Integration stopped at Batch $batch_id"
            error "Check log file for details: $LOG_FILE"
            exit 1
        fi
    done
    
    log ""
    log "=========================================="
    log "Integration Process Completed Successfully"
    log "=========================================="
    log "All $end_batch batches have been integrated"
    log ""
    log "Next steps:"
    log "  1. Review changes: git diff master"
    log "  2. Run final tests: cargo test --all --release"
    log "  3. Merge to master: git checkout master && git merge --no-ff $INTEGRATION_BRANCH"
    log ""
    log "Log file saved to: $LOG_FILE"
    log "=========================================="
}

# Ejecutar función principal con argumentos
main "$@"
