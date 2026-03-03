import subprocess, os
os.environ['GIT_TERMINAL_PROMPT'] = '0'
d = r'C:\Temp\tokmd_fix\b4'

def run(cmd):
    print(f'>>> {cmd}', flush=True)
    r = subprocess.run(cmd, shell=True, cwd=d, stdout=subprocess.PIPE, stderr=subprocess.PIPE, timeout=60)
    out = r.stdout.decode('utf-8', errors='replace').strip()
    err = r.stderr.decode('utf-8', errors='replace').strip()
    if out: print(out, flush=True)
    if err: print(f'ERR: {err}', flush=True)
    print(f'EXIT: {r.returncode}', flush=True)
    return r.returncode

run('git add -A')
run('git diff --cached --name-only')
run('git commit --amend --no-verify -m "fix: formatting and typos" -m "Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>"')
run('git push --force-with-lease origin test/tier2-adapter-expansion')
