import subprocess, os, sys
os.chdir(r"C:\Code\Rust\tokmd")

def run(cmd, timeout=300):
    r = subprocess.run(cmd, capture_output=True, text=True, timeout=timeout)
    line = f"[{'OK' if r.returncode == 0 else 'FAIL'}] {' '.join(cmd)}"
    if r.returncode != 0:
        line += f"\n  OUT: {r.stdout[:300]}\n  ERR: {r.stderr[:300]}"
    return r, line

lines = []

# 1. Reset the bad commit on test/fuzz-expansion
r, l = run(["git", "reset", "--hard", "HEAD~1"]); lines.append(l)

# 2. Stash anything
r, l = run(["git", "stash", "--include-untracked"]); lines.append(l)

# 3. Checkout the correct branch
r, l = run(["git", "checkout", "test/bdd-expansion-wave"]); lines.append(l)
if r.returncode != 0:
    r, l = run(["git", "checkout", "-b", "test/bdd-expansion-wave", "origin/test/bdd-expansion-wave"]); lines.append(l)

# 4. Reset to the remote version
r, l = run(["git", "fetch", "origin", "test/bdd-expansion-wave"]); lines.append(l)
r, l = run(["git", "reset", "--hard", "origin/test/bdd-expansion-wave"]); lines.append(l)

# 5. Verify we're on the right branch
r, l = run(["git", "branch", "--show-current"]); lines.append(l)
branch = r.stdout.strip()
r, l = run(["git", "log", "-1", "--oneline"]); lines.append(l)
head = r.stdout.strip()

if branch != "test/bdd-expansion-wave":
    lines.append(f"FATAL: Wrong branch: {branch}")
    with open("_final_result.txt", "w") as f:
        f.write("\n".join(lines))
    sys.exit(1)

if "a556e90" not in head:
    lines.append(f"WARNING: Unexpected HEAD: {head}")

# 6. Apply normalize_rel_path fix
path_file = os.path.join("crates", "tokmd-path", "src", "lib.rs")
with open(path_file, "r") as f:
    content = f.read()

# Replace the function body
old_body = """    if let Some(stripped) = normalized.strip_prefix("./") {
        stripped.to_string()
    } else {
        normalized
    }"""
new_body = """    let mut s = normalized.as_str();
    while let Some(rest) = s.strip_prefix("./") {
        s = rest;
    }
    s.to_string()"""

if old_body in content:
    content = content.replace(old_body, new_body)
    content = content.replace(
        '/// - strips one leading `./`',
        '/// - strips all leading `./` segments'
    )
    # Add new doctest example
    content = content.replace(
        '/// assert_eq!(normalize_rel_path("../lib.rs"), "../lib.rs");\n/// ```',
        '/// assert_eq!(normalize_rel_path("../lib.rs"), "../lib.rs");\n/// assert_eq!(normalize_rel_path("././src/lib.rs"), "src/lib.rs");\n/// ```'
    )
    with open(path_file, "w") as f:
        f.write(content)
    lines.append("[OK] Applied normalize_rel_path fix")
elif "while let Some(rest)" in content:
    lines.append("[OK] normalize_rel_path already fixed")
else:
    lines.append("[FAIL] Could not find code to fix in lib.rs")
    with open("_final_result.txt", "w") as f:
        f.write("\n".join(lines))
    sys.exit(1)

# 7. Run cargo fmt
r, l = run(["cargo", "fmt", "--all"], timeout=600); lines.append(l)

# 8. Verify fmt passes
r, l = run(["cargo", "fmt", "--all", "--", "--check"], timeout=300); lines.append(l)

# 9. Stage and commit
r, l = run(["git", "add", "crates/"]); lines.append(l)
r, l = run(["git", "diff", "--cached", "--name-status"]); lines.append(l)
lines.append(f"  Staged files: {r.stdout.strip()}")

msg = """fix: resolve rustfmt and normalize_rel_path idempotency issues

- Run cargo fmt to fix formatting across workspace test files
- Fix normalize_rel_path to strip all leading ./ segments (not just one)
  to ensure idempotency for paths like ././src/lib.rs

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>"""

r, l = run(["git", "commit", "-m", msg]); lines.append(l)

# 10. Push
r, l = run(["git", "push", "origin", "test/bdd-expansion-wave"]); lines.append(l)

# 11. Verify final state
r, l = run(["git", "log", "-1", "--oneline"]); lines.append(l)
r, l = run(["git", "branch", "--show-current"]); lines.append(l)

with open("_final_result.txt", "w") as f:
    f.write("\n".join(lines))
