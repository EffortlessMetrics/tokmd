import subprocess, os
os.chdir(r"C:\Code\Rust\tokmd")
log = []

def run(cmd, timeout=120):
    r = subprocess.run(cmd, capture_output=True, text=True, timeout=timeout)
    log.append(f">>> {' '.join(cmd)}\nEXIT: {r.returncode}\nOUT: {r.stdout[:1000]}\nERR: {r.stderr[:1000]}\n")
    return r

run(["git", "log", "-1", "--oneline"])
run(["git", "diff", "--name-status"])
run(["git", "add", "crates/"])
run(["git", "diff", "--cached", "--name-status"])

msg = """fix: resolve rustfmt and normalize_rel_path idempotency issues

- Run cargo fmt to fix formatting across workspace test files
- Fix normalize_rel_path to strip all leading ./ segments (not just one)
  to ensure idempotency for paths like ././src/lib.rs

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>"""
run(["git", "commit", "-m", msg])
run(["git", "push", "origin", "test/bdd-expansion-wave"])
run(["git", "log", "-1", "--oneline"])

with open("_commit_result.txt", "w") as f:
    f.write("\n".join(log))
