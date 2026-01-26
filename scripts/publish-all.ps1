# scripts/publish-all.ps1
# Automates the release of all tokmd crates in the correct dependency order.
# Usage: ./scripts/publish-all.ps1 [-DryRun]

param (
    [switch]$DryRun
)

$crates = @(
    "tokmd-types",
    "tokmd-config",
    "tokmd-model",
    "tokmd-format",
    "tokmd-scan",
    "tokmd-tokeignore",
    "tokmd-core",
    "tokmd"
)

Write-Host "🚀 Starting automated publish sequence..." -ForegroundColor Cyan
if ($DryRun) {
    Write-Host "Running in DRY RUN mode." -ForegroundColor Yellow
}

foreach ($crate in $crates) {
    Write-Host "📦 Publishing $crate..." -ForegroundColor Green
    
    $args = @("publish", "-p", $crate)
    if ($DryRun) {
        $args += "--dry-run"
    }

    $process = Start-Process -FilePath "cargo" -ArgumentList $args -PassThru -NoNewWindow -Wait
    
    if ($process.ExitCode -ne 0) {
        Write-Host "❌ Failed to publish $crate. Aborting." -ForegroundColor Red
        exit 1
    }
    
    # Wait a bit for crates.io index to update (skip in dry run)
    if (-not $DryRun) {
        Write-Host "Sleeping 10s for crates.io propagation..." -ForegroundColor Gray
        Start-Sleep -Seconds 10
    }
}

Write-Host "✅ All crates published successfully!" -ForegroundColor Cyan
