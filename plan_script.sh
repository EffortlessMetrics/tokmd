#!/bin/bash
for file in crates/*/tests/*.rs; do
    total_tests=$(grep -c "fn " $file)
    git_gated=$(grep -c "#\[cfg(feature" $file)
    if [ "$git_gated" -ge 10 ]; then
        echo "MATCH: File: $file has $git_gated #cfg(feature) out of $total_tests total fns."
    fi
done
