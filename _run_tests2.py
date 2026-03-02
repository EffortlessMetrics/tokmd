import subprocess, os
os.chdir(r"C:\Code\Rust\tokmd")

# Test 1: tokmd-path normalize_rel_path
r = subprocess.run(["cargo", "test", "-p", "tokmd-path", "--", "normalize_rel_path"],
                    capture_output=True, text=True, timeout=600)
with open("_test_path.txt", "w") as f:
    f.write(f"EXIT: {r.returncode}\n{r.stderr[-2000:]}\n")

# Test 2: determinism_props idempotent
r = subprocess.run(["cargo", "test", "-p", "tokmd-types", "--test", "determinism_props",
                     "--", "normalize_rel_path_is_idempotent"],
                    capture_output=True, text=True, timeout=600)
with open("_test_idempotent.txt", "w") as f:
    f.write(f"EXIT: {r.returncode}\n{r.stderr[-2000:]}\n")
