# scripts/publish-all.ps1
# DEPRECATED: Use `cargo xtask publish` instead.
#
# The new xtask provides:
# - Automatic dependency ordering from cargo metadata
# - Pre-publish checks (git, version, changelog, tests)
# - Retry logic for crates.io propagation delays
# - --dry-run validation (real `cargo publish --dry-run`)
# - --plan mode to preview before executing
# - TTY detection and --yes flag for CI
#
# Migration:
#   ./scripts/publish-all.ps1 -DryRun  →  cargo xtask publish --dry-run
#   ./scripts/publish-all.ps1          →  cargo xtask publish --yes
#
# Original usage (deprecated): ./scripts/publish-all.ps1 [-DryRun]

param (
    [switch]$DryRun
)

# =============================================================================
# DEPRECATION WARNING - This script is deprecated and will be removed.
# =============================================================================
Write-Host ""
Write-Host "╔══════════════════════════════════════════════════════════════════╗" -ForegroundColor Yellow
Write-Host "║  DEPRECATED: This script is deprecated. Use xtask instead:       ║" -ForegroundColor Yellow
Write-Host "║                                                                  ║" -ForegroundColor Yellow
Write-Host "║    cargo xtask publish --plan      # Preview publish order       ║" -ForegroundColor Yellow
Write-Host "║    cargo xtask publish --dry-run   # Validate packaging          ║" -ForegroundColor Yellow
Write-Host "║    cargo xtask publish --yes       # Publish all crates          ║" -ForegroundColor Yellow
Write-Host "║                                                                  ║" -ForegroundColor Yellow
Write-Host "╚══════════════════════════════════════════════════════════════════╝" -ForegroundColor Yellow
Write-Host ""
Write-Host "Continuing in 5 seconds... (Ctrl+C to abort)" -ForegroundColor Gray
Start-Sleep -Seconds 5

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
