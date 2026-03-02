files = [
    'crates/tokmd-badge/tests/snapshots.rs',
    'crates/tokmd-fun/tests/snapshots.rs',
    'crates/tokmd-format/tests/snapshots.rs',
    'crates/tokmd-analysis-format/tests/snapshot_golden.rs',
    'crates/tokmd-analysis-fun/tests/snapshot_eco.rs',
]
for f in files:
    c = open(f).read()
    print(f'{f}: {c.count("#[test]")} tests')
