#!/usr/bin/env pwsh
Set-Location 'C:\Code\Rust\tokmd-analysis-tests2'

Write-Host "=== Running cargo fmt-fix ===" -ForegroundColor Green
cargo fmt-fix
$fmtExitCode = $LASTEXITCODE

Write-Host "`n=== Running cargo clippy ===" -ForegroundColor Green
cargo clippy -p tokmd-analysis-halstead -p tokmd-analysis-license -p tokmd-analysis-topics -p tokmd-analysis-fingerprint -- -D warnings
$clippyExitCode = $LASTEXITCODE

Write-Host "`n=== Summary ===" -ForegroundColor Green
Write-Host "cargo fmt-fix exit code: $fmtExitCode"
Write-Host "cargo clippy exit code: $clippyExitCode"

if ($fmtExitCode -eq 0 -and $clippyExitCode -eq 0) {
    Write-Host "`nAll checks passed!" -ForegroundColor Green
} else {
    Write-Host "`nSome checks failed." -ForegroundColor Red
}
