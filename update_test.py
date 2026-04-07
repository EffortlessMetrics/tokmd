import sys

content = open('crates/tokmd-analysis-types/tests/baseline_fallback.rs', 'r').read()

search = """\
        "integrity": { "algo": "", "hash": "", "entries": [] }"""

replace = """\
        "integrity": { "algo": "", "hash": "", "entries": 0 }"""

if search in content:
    content = content.replace(search, replace)
    with open('crates/tokmd-analysis-types/tests/baseline_fallback.rs', 'w') as f:
        f.write(content)
    print("Updated test file for integrity.entries.")
else:
    print("Could not find search string.")
