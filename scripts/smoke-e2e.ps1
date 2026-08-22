# LotusOS end-to-end smoke test against the demo fixture.
#
# Usage:  powershell -File scripts\smoke-e2e.ps1
#
# Requires: cargo build -p lotus-cli
# Isolation: uses its own LOTUS_HOME so your real workspace state is untouched.

$ErrorActionPreference = "Stop"

$repoRoot = Split-Path -Parent $PSScriptRoot
$lotus = Join-Path $repoRoot "target\debug\lotus.exe"
if (-not (Test-Path $lotus)) {
    Write-Error "target\debug\lotus.exe not found. Run: cargo build -p lotus-cli"
}

$env:LOTUS_HOME = Join-Path $env:TEMP ("lotus-smoke-" + [guid]::NewGuid().ToString("N").Substring(0, 8))
Write-Host "LOTUS_HOME = $env:LOTUS_HOME"
Write-Host ""

function Show([string]$title) { Write-Host "`n=== $title ===" }

Show "add (with trust)"
& $lotus add (Join-Path $repoRoot "fixtures\demo-workspace") --trust
if ($LASTEXITCODE -ne 0) { exit 1 }

Show "start"
& $lotus start demo
if ($LASTEXITCODE -ne 0) { exit 1 }
Start-Sleep -Seconds 3

Show "status"
& $lotus status demo
if ($LASTEXITCODE -ne 0) { exit 1 }

Show "doctor"
& $lotus doctor demo
if ($LASTEXITCODE -ne 0) { Write-Error "doctor reported problems" }

Show "logs"
& $lotus logs demo --lines 5

Show "events"
& $lotus events demo --limit 6

Show "checkpoint + restore"
& $lotus checkpoint demo --note "smoke" | Out-Null
$cpLine = (& $lotus checkpoints demo | Select-Object -First 1)
$cpId = ($cpLine -split '\s+')[0]
& $lotus restore demo $cpId
if ($LASTEXITCODE -ne 0) { exit 1 }
Start-Sleep -Seconds 3
& $lotus status demo

Show "stop"
& $lotus stop demo
if ($LASTEXITCODE -ne 0) { exit 1 }

Show "final list"
& $lotus list

Write-Host "`nSMOKE TEST PASSED" -ForegroundColor Green
