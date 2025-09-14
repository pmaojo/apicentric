#!/usr/bin/env pwsh
$ErrorActionPreference = 'Stop'

$repoUrl = 'https://github.com/pulse-1/mockforge/releases/latest/download'
$os = 'windows'
$arch = [System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture.ToString().ToLower()
switch ($arch) {
  'x64' { $arch = 'x86_64' }
  'arm64' { $arch = 'arm64' }
  default { Write-Error "Unsupported architecture: $arch"; exit 1 }
}

$file = "mockforge-$os-$arch.zip"
$tempDir = Join-Path ([System.IO.Path]::GetTempPath()) ([System.Guid]::NewGuid())
New-Item -ItemType Directory -Path $tempDir | Out-Null
$archive = Join-Path $tempDir $file
Invoke-WebRequest -Uri "$repoUrl/$file" -OutFile $archive
Expand-Archive -Path $archive -DestinationPath $tempDir -Force

$dest = Join-Path $env:UserProfile 'bin'
if (-not (Test-Path $dest)) { New-Item -ItemType Directory -Path $dest | Out-Null }
Move-Item -Path (Join-Path $tempDir 'mockforge.exe') -Destination (Join-Path $dest 'mockforge.exe') -Force
Write-Host "mockforge installed to $dest"

