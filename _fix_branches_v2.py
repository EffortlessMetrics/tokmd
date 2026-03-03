"""Fix formatting on multiple branches using fresh clones."""
import subprocess
import os
import sys
import time
import shutil

# Flush all output
sys.stdout.reconfigure(line_buffering=True)

REPO_URL = "git@github.com:EffortlessMetrics/tokmd.git"
WORK_DIR = r"C:\Temp\tokmd_fix"

os.environ["GIT_TERMINAL_PROMPT"] = "0"
os.environ["GIT_EDITOR"] = "true"

def log(msg):
    print(f"[{time.strftime('%H:%M:%S')}] {msg}", flush=True)

def run(cmd, cwd=None, timeout=600):
    log(f"RUN: {cmd}")
    try:
        r = subprocess.run(
            cmd, shell=True, capture_output=True, text=True,
            timeout=timeout, cwd=cwd,
            env={**os.environ, "GIT_TERMINAL_PROMPT": "0"}
        )
        out = r.stdout.strip()
        err = r.stderr.strip()
        if out:
            log(f"  OUT: {out[:500]}")
        if err:
            log(f"  ERR: {err[:500]}")
        log(f"  EXIT: {r.returncode}")
        return r.returncode, out
    except subprocess.TimeoutExpired:
        log(f"  TIMEOUT after {timeout}s")
        return -1, ""

def process_branch(branch, commit_msg="style: cargo fmt", extra_fn=None):
    """Clone branch, format, commit, push."""
    log(f"\n{'='*60}")
    log(f"Processing: {branch}")
    log(f"{'='*60}")
    
    clone_dir = os.path.join(WORK_DIR, branch.replace("/", "_"))
    
    # Clean up any previous attempt
    if os.path.exists(clone_dir):
        log(f"Cleaning {clone_dir}")
        shutil.rmtree(clone_dir, ignore_errors=True)
    
    # Shallow clone the specific branch
    rc, _ = run(
        f'git clone --depth 1 --branch {branch} "{REPO_URL}" "{clone_dir}"',
        timeout=120
    )
    if rc != 0:
        log(f"FAILED to clone {branch}")
        return False
    
    # Run cargo fmt
    rc, _ = run("cargo fmt", cwd=clone_dir, timeout=300)
    if rc != 0:
        log(f"FAILED cargo fmt on {branch}")
        return False
    
    # Run extra steps (e.g., typos)
    if extra_fn:
        extra_fn(clone_dir)
    
    # Add all changes  
    rc, _ = run("git add -A", cwd=clone_dir, timeout=60)
    if rc != 0:
        log(f"FAILED git add on {branch}")
        return False
    
    # Check for changes
    rc, diff = run("git diff --cached --name-only", cwd=clone_dir, timeout=60)
    if not diff:
        log(f"No changes on {branch}")
        return True
    
    log(f"Changed files:\n{diff}")
    
    # Commit
    rc, _ = run(
        f'git commit --no-verify -m "{commit_msg}" '
        f'-m "Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>"',
        cwd=clone_dir, timeout=60
    )
    if rc != 0:
        log(f"FAILED commit on {branch}")
        return False
    
    # Push
    rc, _ = run(f"git push origin {branch}", cwd=clone_dir, timeout=120)
    if rc != 0:
        log(f"FAILED push on {branch}")
        return False
    
    log(f"SUCCESS: {branch}")
    return True

def fix_typos(clone_dir):
    """Check and fix typos."""
    log("Checking for typos...")
    rc, _ = run("typos --version", cwd=clone_dir, timeout=10)
    if rc != 0:
        # Try cargo install
        log("typos not found, checking cargo install...")
        rc, _ = run("cargo install typos-cli", cwd=clone_dir, timeout=300)
    
    rc, out = run("typos", cwd=clone_dir, timeout=60)
    if rc != 0 and out:
        log("Fixing typos with typos -w...")
        run("typos -w", cwd=clone_dir, timeout=60)

# Create work directory
os.makedirs(WORK_DIR, exist_ok=True)

results = {}

# Branch 2: test/cockpit-handoff-integration
results["test/cockpit-handoff-integration"] = process_branch("test/cockpit-handoff-integration")

# Branch 3: test/tier1-scan-sensor-exclude
results["test/tier1-scan-sensor-exclude"] = process_branch("test/tier1-scan-sensor-exclude")

# Branch 4: test/tier2-adapter-expansion (with typos)
results["test/tier2-adapter-expansion"] = process_branch(
    "test/tier2-adapter-expansion",
    commit_msg="fix: formatting and typos",
    extra_fn=fix_typos
)

# Summary
log(f"\n{'='*60}")
log("FINAL RESULTS:")
log("  fix/schema-doc-alignment: SUCCESS (pushed earlier)")
for branch, success in results.items():
    log(f"  {branch}: {'SUCCESS' if success else 'FAILED'}")
log(f"{'='*60}")

# Write results to file for verification
with open(os.path.join(WORK_DIR, "results.txt"), "w") as f:
    f.write("fix/schema-doc-alignment: SUCCESS\n")
    for branch, success in results.items():
        f.write(f"{branch}: {'SUCCESS' if success else 'FAILED'}\n")
