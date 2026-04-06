#!/bin/bash
NEW_ENTRY='{
  "run_id": "b2ba044e-893b-4117-8507-bba537f06de6",
  "timestamp": "'$(date -u +"%Y-%m-%dT%H:%M:%SZ")'",
  "description": "Fix pyo3 extension module linking error during all-features test"
}'
jq --argjson entry "$NEW_ENTRY" '. += [$entry]' .jules/compat/ledger.json > temp.json && mv temp.json .jules/compat/ledger.json
