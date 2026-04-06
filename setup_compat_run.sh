#!/bin/bash
mkdir -p .jules/compat/envelopes/
mkdir -p .jules/compat/runs/

RUN_ID=$(cat /proc/sys/kernel/random/uuid)
echo $RUN_ID > .jules/compat/current_run_id

RUN_DATE=$(date -u +"%Y-%m-%d")
echo $RUN_DATE > .jules/compat/current_run_date

echo "{ \"run_id\": \"$RUN_ID\", \"timestamp\": \"$(date -Iseconds)\", \"description\": \"Apply inner git feature gate to integration tests in tokmd\" }" > .jules/compat/envelopes/${RUN_ID}.json

echo "Run envelope created: .jules/compat/envelopes/${RUN_ID}.json"
