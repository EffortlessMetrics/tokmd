import os
path = r'C:\Code\Rust\tokmd\crates\tokmd-badge\tests\snapshots.rs'
with open(path, 'r') as f: content = f.read()
print('before:', content.count('#[test]'))
marker = '// \u2500\u2500 Property: badge always'
print('marker found:', marker in content)
