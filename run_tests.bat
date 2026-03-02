@echo off
cd /d "C:\Code\Rust\tokmd-analysis-tests2"
cargo test -p tokmd-analysis-halstead -p tokmd-analysis-license -p tokmd-analysis-topics -p tokmd-analysis-fingerprint --verbose 2>&1 > test_results.txt
echo Test completed, results in test_results.txt
