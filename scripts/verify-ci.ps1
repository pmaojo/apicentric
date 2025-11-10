# CI Verification Script for Windows
# This script runs the same checks that CI runs to verify everything works locally

$ErrorActionPreference = "Continue"

Write-Host "🔍 Apicentric CI Verification Script" -ForegroundColor Cyan
Write-Host "====================================" -ForegroundColor Cyan
Write-Host ""

# Track failures
$script:Failures = 0

# Function to run a check
function Run-Check {
    param(
        [string]$Name,
        [string]$Command
    )

    Write-Host "Running: $Name" -ForegroundColor Yellow

    $result = Invoke-Expression $Command
    $exitCode = $LASTEXITCODE

    if ($exitCode -eq 0) {
        Write-Host "✓ $Name passed" -ForegroundColor Green
        Write-Host ""
        return $true
    } else {
        Write-Host "✗ $Name failed" -ForegroundColor Red
        Write-Host ""
        $script:Failures++
        return $false
    }
}

# Function to verify checksums
function Verify-Checksums {
    param(
        [string]$FilePath,
        [string]$ExpectedHash
    )

    if (!(Test-Path $FilePath)) {
        Write-Host "File not found: $FilePath" -ForegroundColor Red
        return $false
    }

    try {
        $actualHash = Get-FileHash -Path $FilePath -Algorithm SHA256 | Select-Object -ExpandProperty Hash
        if ($actualHash -eq $ExpectedHash.ToUpper()) {
            Write-Host "✓ Checksum verified for $FilePath" -ForegroundColor Green
            return $true
        } else {
            Write-Host "✗ Checksum mismatch for $FilePath" -ForegroundColor Red
            Write-Host "  Expected: $ExpectedHash" -ForegroundColor Red
            Write-Host "  Actual:   $actualHash" -ForegroundColor Red
            return $false
        }
    } catch {
        Write-Host "✗ Error verifying checksum for $FilePath" -ForegroundColor Red
        return $false
    }
}

# 1. Format Check
Write-Host "📝 Step 1: Format Check" -ForegroundColor Cyan
Write-Host "----------------------" -ForegroundColor Cyan
Run-Check "cargo fmt --check" "cargo fmt --all -- --check"

# 2. Lint Check
Write-Host "🔎 Step 2: Lint Check" -ForegroundColor Cyan
Write-Host "--------------------" -ForegroundColor Cyan
Run-Check "cargo clippy" "cargo clippy --all-targets --all-features -- -D warnings"

# 3. Build Tests
Write-Host "🔨 Step 3: Build Tests" -ForegroundColor Cyan
Write-Host "---------------------" -ForegroundColor Cyan

Run-Check "Minimal build" "cargo build --no-default-features --features minimal"
Run-Check "Default build" "cargo build"
Run-Check "CLI tools build" "cargo build --features cli-tools"
Run-Check "Full build" "cargo build --all-features"

# 4. Test Suites
Write-Host "🧪 Step 4: Test Suites" -ForegroundColor Cyan
Write-Host "---------------------" -ForegroundColor Cyan

Run-Check "Tests (minimal)" "cargo test --no-default-features --features minimal"
Run-Check "Tests (default)" "cargo test"
Run-Check "Tests (full)" "cargo test --all-features"

# 5. Security Audit
Write-Host "🔒 Step 5: Security Audit" -ForegroundColor Cyan
Write-Host "------------------------" -ForegroundColor Cyan

$auditInstalled = Get-Command cargo-audit -ErrorAction SilentlyContinue
if ($auditInstalled) {
    Run-Check "cargo audit" "cargo audit"
} else {
    Write-Host "⚠ cargo-audit not installed, skipping" -ForegroundColor Yellow
    Write-Host "  Install with: cargo install cargo-audit"
    Write-Host ""
}

# 6. Checksum Verification
Write-Host "🔐 Step 6: Checksum Verification" -ForegroundColor Cyan
Write-Host "-------------------------------" -ForegroundColor Cyan

# Verify checksums for downloaded binaries (if any)
$checksumsValid = $true
$checksumFiles = Get-ChildItem -Path "." -Filter "*.sha256" -Recurse -ErrorAction SilentlyContinue

foreach ($checksumFile in $checksumFiles) {
    $checksumContent = Get-Content $checksumFile.FullName -Raw
    # Parse checksum file format: "hash filename"
    $lines = $checksumContent -split "`n" | Where-Object { $_ -match '\S' }
    foreach ($line in $lines) {
        if ($line -match '^([a-fA-F0-9]{64})\s+(.+)$') {
            $expectedHash = $matches[1]
            $fileName = $matches[2].Trim()
            $filePath = Join-Path $checksumFile.DirectoryName $fileName
            if (!(Verify-Checksums -FilePath $filePath -ExpectedHash $expectedHash)) {
                $checksumsValid = $false
            }
        }
    }
}

if ($checksumsValid) {
    Write-Host "✓ All checksums verified successfully" -ForegroundColor Green
    Write-Host ""
} else {
    Write-Host "✗ Checksum verification failed" -ForegroundColor Red
    Write-Host ""
    $script:Failures++
}

# 7. Documentation Check
Write-Host "📚 Step 7: Documentation Check" -ForegroundColor Cyan
Write-Host "------------------------------" -ForegroundColor Cyan

Run-Check "cargo doc" "cargo doc --all-features --no-deps"

# Summary
Write-Host "==================================" -ForegroundColor Cyan
Write-Host "📊 Verification Summary" -ForegroundColor Cyan
Write-Host "==================================" -ForegroundColor Cyan

if ($script:Failures -eq 0) {
    Write-Host "✓ All checks passed!" -ForegroundColor Green
    Write-Host ""
    Write-Host "Your code is ready for CI. All checks that run in GitHub Actions"
    Write-Host "have passed locally."
    exit 0
} else {
    Write-Host "✗ $($script:Failures) check(s) failed" -ForegroundColor Red
    Write-Host ""
    Write-Host "Please fix the failing checks before pushing to CI."
    exit 1
}
