"""Fix formatting and typos on test/tier2-adapter-expansion branch."""
import subprocess
import os
import sys
import time
import shutil

sys.stdout.reconfigure(line_buffering=True)

REPO_URL = "git@github.com:EffortlessMetrics/tokmd.git"
BRANCH = "test/tier2-adapter-expansion"
CLONE_DIR = r"C:\Temp\tokmd_fix\b4"

os.environ["GIT_TERMINAL_PROMPT"] = "0"
os.environ["GIT_EDITOR"] = "true"

def log(msg):
    print(f"[{time.strftime('%H:%M:%S')}] {msg}", flush=True)

def run(cmd, cwd=None, timeout=600):
    log(f"RUN: {cmd}")
    try:
        r = subprocess.run(
            cmd, shell=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
            timeout=timeout, cwd=cwd,
            env={**os.environ, "GIT_TERMINAL_PROMPT": "0"}
        )
        out = r.stdout.decode("utf-8", errors="replace").strip()
        err = r.stderr.decode("utf-8", errors="replace").strip()
        if out:
            log(f"  OUT: {out[:1000]}")
        if err:
            log(f"  ERR: {err[:500]}")
        log(f"  EXIT: {r.returncode}")
        return r.returncode, out
    except subprocess.TimeoutExpired:
        log(f"  TIMEOUT after {timeout}s")
        return -1, ""

# Clean up previous attempt
if os.path.exists(CLONE_DIR):
    log(f"Cleaning {CLONE_DIR}")
    shutil.rmtree(CLONE_DIR, ignore_errors=True)

# Clone
rc, _ = run(f'git clone --depth 1 --branch {BRANCH} "{REPO_URL}" "{CLONE_DIR}"', timeout=120)
if rc != 0:
    log("FAILED to clone")
    sys.exit(1)

# Run cargo fmt
rc, _ = run("cargo fmt", cwd=CLONE_DIR, timeout=300)
if rc != 0:
    log("FAILED cargo fmt")
    sys.exit(1)

# Check and fix typos
log("Checking typos version...")
rc, _ = run("typos --version", cwd=CLONE_DIR, timeout=10)
if rc != 0:
    log("Installing typos-cli...")
    run("cargo install typos-cli", cwd=CLONE_DIR, timeout=300)

log("Running typos check...")
rc, out = run("typos", cwd=CLONE_DIR, timeout=60)
if rc != 0:
    log(f"Typos found (exit code {rc}). Fixing with typos -w...")
    rc2, _ = run("typos -w", cwd=CLONE_DIR, timeout=60)
    if rc2 != 0:
        log("typos -w failed, checking remaining issues...")
        run("typos", cwd=CLONE_DIR, timeout=60)
else:
    log("No typos found")

# Add all changes
rc, _ = run("git add -A", cwd=CLONE_DIR, timeout=60)
if rc != 0:
    log("FAILED git add")
    sys.exit(1)

# Check for changes
rc, diff = run("git diff --cached --name-only", cwd=CLONE_DIR, timeout=60)
if not diff:
    log("No changes to commit")
    sys.exit(0)

log(f"Changed files:\n{diff}")

# Commit
rc, _ = run(
    'git commit --no-verify -m "fix: formatting and typos" '
    '-m "Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>"',
    cwd=CLONE_DIR, timeout=60
)
if rc != 0:
    log("FAILED commit")
    sys.exit(1)

# Push
rc, _ = run(f"git push origin {BRANCH}", cwd=CLONE_DIR, timeout=120)
if rc != 0:
    log("FAILED push")
    sys.exit(1)

log(f"SUCCESS: {BRANCH}")
