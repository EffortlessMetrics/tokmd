import subprocess, os
os.chdir(r"C:\Code\Rust\tokmd")
r = subprocess.run(["cargo", "test", "-p", "tokmd-path", "--test", "bdd"],
                    capture_output=True, text=True, timeout=300)
with open("_test_bdd2.txt", "w") as f:
    f.write(f"EXIT: {r.returncode}\n")
    f.write(f"STDERR:\n{r.stderr[-2000:]}\n")
    f.write(f"STDOUT:\n{r.stdout[-1000:]}\n")
