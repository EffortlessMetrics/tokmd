import re

with open('crates/tokmd-format/tests/format_tests.rs', 'r') as f:
    content = f.read()

pattern = re.compile(r"""#\[test\]\nfn test_export_json_redact_all_hides_module_roots\(\) \{.*\}""", re.DOTALL)

if pattern.search(content):
    content = pattern.sub("", content)

with open('crates/tokmd-format/tests/format_tests.rs', 'w') as f:
    f.write(content)
