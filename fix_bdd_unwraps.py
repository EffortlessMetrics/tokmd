import os
import re

def update_file(filepath):
    with open(filepath, 'r') as f:
        content = f.read()

    content = re.sub(
        r'\.unwrap\(\)',
        r'.expect("Test fixture expectations must be infallible")',
        content
    )

    with open(filepath, 'w') as f:
        f.write(content)

for root, _, files in os.walk('crates/tokmd/tests/'):
    for file in files:
        if file.startswith('bdd_') and file.endswith('.rs'):
            update_file(os.path.join(root, file))
