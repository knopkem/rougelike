---
name: implement-all
description: Implement all triaged issue briefs in one go — one fresh opencode session per issue, commit each, then move on. Use when the user wants to work through a numbered backlog of bug briefs (e.g. issues 01..24) without blowing the local LLM's context limit.
---

# Implement All Issues (one fresh session each)

Process a numbered backlog of issue briefs sequentially. **The critical constraint:
never process more than one issue per session.** Each issue gets its own fresh
`opencode run` subprocess — this is the "clear context" step, and it is
mandatory on a local LLM with a ~100K context limit. One session for 24 issues
would exhaust context and destroy output quality.

## How it works

The work is driven by `scripts/implement-all.sh` in the project (it is the
authoritative implementation — do not re-implement its logic inline):

```bash
./scripts/implement-all.sh            # start at issue 01
./scripts/implement-all.sh 10         # start at issue 10
RESUME=1 ./scripts/implement-all.sh   # re-run; skips briefs marked "Status: done"
```

For each issue file (sorted numerically, e.g. `01-hiscore-double-record.md`):

1. **Skip** if the brief's header already says `Status: done`.
2. Capture the current `HEAD`.
3. Spawn a **fresh headless session**:
   `opencode run --dangerously-skip-permissions -m <model> --variant Medium
   "Implement the issue described in <path>. Follow AGENTS.md. Run the test
   suite before committing. Commit referencing the issue. If ambiguous, stop
   without committing."`
   - No `-c`/`-s` flags → brand-new session, clean context.
   - `--dangerously-skip-permissions` (yolo agent) so headless runs never hang
     on a permission prompt.
4. After the run: verify a new commit exists vs the captured HEAD. Log
   COMMITTED / WARNING / FAILED per issue to `.scratch/implement-all.log`.
5. Move to the next issue. The per-issue log makes the batch resumable.

## Usage from the TUI

```
/implement-all            # process everything
/implement-all 10         # start from issue 10
```

If the user asks for a single issue, just run the normal `/implement N` flow —
this skill is for *batches*.

## Configuration (env vars)

| Var | Default | Purpose |
|---|---|---|
| `ISSUES_DIR` | `.scratch/deepdelve/issues` | where numbered briefs live |
| `MODEL` | `qwen-local/Qwen3.8-27B-IQ3_XXS` | opencode model per issue |
| `VARIANT` | `Medium` | reasoning variant (Medium = brief thinking, good for bug fixes) |

## Pitfalls

- **Never loop inside your own session.** If you (the agent reading this skill)
  iterate over 24 issues in the current conversation, you recreate the context
  blowup this skill exists to prevent. Always delegate each issue to a fresh
  `opencode run` subprocess.
- **Do not commit on the user's behalf outside the driver.** Let the subprocess
  commit; the driver verifies and logs. If no commit appeared, tell the user
  which issue needs manual review.
- **Context checkpoints / cache-reuse are server-side concerns** — do not
  instruct the user to change llama-server flags for this workflow.
- If a run fails (non-zero exit), continue with the next issue and report the
  failure at the end — a single flaky issue should not block the batch.
