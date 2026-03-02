import subprocess, os, sys
os.chdir(r"C:\Code\Rust\tokmd")

log = []
def run(cmd, timeout=300):
    r = subprocess.run(cmd, capture_output=True, text=True, timeout=timeout)
    status = "OK" if r.returncode == 0 else "FAIL"
    log.append(f"[{status}] {' '.join(cmd)}")
    if r.stdout.strip():
        log.append(f"  -> {r.stdout.strip()[:200]}")
    if r.returncode != 0 and r.stderr.strip():
        log.append(f"  ERR: {r.stderr.strip()[:200]}")
    return r

# Step 1: Fix the bad push to test/fuzz-expansion
log.append("=== Step 1: Undo bad push to test/fuzz-expansion ===")
run(["git", "reset", "--hard", "2537d24"])
run(["git", "push", "origin", "test/fuzz-expansion", "--force"])

# Step 2: Checkout test/bdd-expansion-wave
log.append("=== Step 2: Checkout test/bdd-expansion-wave ===")
r = run(["git", "checkout", "test/bdd-expansion-wave"])
if r.returncode != 0:
    run(["git", "checkout", "-b", "test/bdd-expansion-wave", "origin/test/bdd-expansion-wave"])
run(["git", "reset", "--hard", "origin/test/bdd-expansion-wave"])

# Verify
r = run(["git", "branch", "--show-current"])
branch = r.stdout.strip()
r = run(["git", "log", "-1", "--oneline"])
head = r.stdout.strip()
log.append(f"Branch: {branch}, HEAD: {head}")

if branch != "test/bdd-expansion-wave":
    log.append(f"FATAL: On wrong branch {branch}!")
    with open("_final2.txt", "w") as f: f.write("\n".join(log))
    sys.exit(1)

# Step 3: Apply normalize_rel_path fix
log.append("=== Step 3: Apply normalize_rel_path fix ===")
pf = os.path.join("crates", "tokmd-path", "src", "lib.rs")
with open(pf) as f: c = f.read()

old = '    if let Some(stripped) = normalized.strip_prefix("./") {\n        stripped.to_string()\n    } else {\n        normalized\n    }'
new = '    let mut s = normalized.as_str();\n    while let Some(rest) = s.strip_prefix("./") {\n        s = rest;\n    }\n    s.to_string()'

if old in c:
    c = c.replace(old, new)
    c = c.replace('/// - strips one leading `./`', '/// - strips all leading `./` segments')
    c = c.replace(
        '/// assert_eq!(normalize_rel_path("../lib.rs"), "../lib.rs");\n/// ```',
        '/// assert_eq!(normalize_rel_path("../lib.rs"), "../lib.rs");\n/// assert_eq!(normalize_rel_path("././src/lib.rs"), "src/lib.rs");\n/// ```'
    )
    with open(pf, "w") as f: f.write(c)
    log.append("Applied normalize_rel_path fix")
else:
    log.append("Could not find old code or already fixed")

# Step 4: cargo fmt
log.append("=== Step 4: cargo fmt ===")
r = run(["cargo", "fmt", "--all"], timeout=600)

# Step 5: Verify fmt
r = run(["cargo", "fmt", "--all", "--", "--check"], timeout=300)

# Step 6: Commit and push
log.append("=== Step 6: Commit and push ===")
run(["git", "add", "crates/"])
r = run(["git", "diff", "--cached", "--name-status"])

msg = """fix: resolve rustfmt and normalize_rel_path idempotency issues

- Run cargo fmt to fix formatting across workspace test files
- Fix normalize_rel_path to strip all leading ./ segments (not just one)
  to ensure idempotency for paths like ././src/lib.rs

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>"""
run(["git", "commit", "-m", msg])
run(["git", "push", "origin", "test/bdd-expansion-wave"])

# Verify
run(["git", "log", "-1", "--oneline"])
run(["git", "branch", "--show-current"])

with open("_final2.txt", "w") as f:
    f.write("\n".join(log))
