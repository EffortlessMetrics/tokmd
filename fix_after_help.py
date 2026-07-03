import re
import os

path = 'crates/tokmd/src/cli/parser/context.rs'

with open(path, 'r') as f:
    content = f.read()

# tokmd handoff . --review-packet-dir .tokmd/review --proof-route target/ci/proof-pack-route.json --proof-plan target/proof/proof-plan.json
content = content.replace(
    r'Examples:\n  tokmd handoff crates/tokmd xtask --out-dir .handoff --budget 128k\n  tokmd handoff . --review-packet-dir .tokmd/review --proof-route target/ci/proof-pack-route.json --proof-plan target/proof/proof-plan.json',
    r'Examples:\n  tokmd handoff crates/tokmd xtask --out-dir .handoff --budget 128k\n  tokmd handoff --no-git\n  tokmd handoff . --review-packet-dir .tokmd/review --proof-route target/ci/proof-pack-route.json --proof-plan target/proof/proof-plan.json'
)

with open(path, 'w') as f:
    f.write(content)
