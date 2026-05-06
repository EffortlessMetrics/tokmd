# Option A: Create a Friction Item for the Superseded Work
Since the reviewer closed this PR as superseded by #1606, I should create a learning PR documenting the friction.
I will generate standard run artifacts and a new friction item (in `.jules/friction/open/`) documenting the workflow edge case.
The memory instruction says:
"If an intended patch is found to be superseded by another merged PR during execution, gracefully abort the redundant fix and create a 'learning PR'. This involves generating the standard run artifacts and a new friction item (in `.jules/friction/open/`) documenting the workflow edge case."

Wait, the prompt asked to: "summarize per-run packets into generated indexes/rollups".
And it's superseded. So I'll just write a new learning PR body, reset the code changes (except for my newly created artifacts and friction item), and submit a learning PR!

Wait, `archivist_jules` is already my current run id.
I will create a friction item: `.jules/friction/open/FRIC-$(date +%Y%m%d)-999.md` about being superseded.

First, let's reset the codebase (but keep my `.jules/runs/archivist_jules/` artifacts). Wait, using `git reset --hard origin/main` would wipe out the `.jules/runs/archivist_jules/` too if it wasn't tracked. I'll just `git restore .jules/bin/build_index.py` to revert the patch, and I'll keep the `.jules/runs/` subdirectories. Wait, if it's superseded, the code is ALREADY merged on `main`? No, I am on a fresh clone.
Let me just restore the file I modified: `git restore .jules/bin/build_index.py`, and remove the `FRICTION_ROLLUP.md` file. Wait, I'll keep the index files as they were originally.

Let's do:
`git restore .jules/bin/build_index.py`
`rm .jules/index/generated/FRICTION_ROLLUP.md`

Then I will create the friction item:
