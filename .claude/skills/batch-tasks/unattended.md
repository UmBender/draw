# Unattended batch session (skill: batch-tasks)

You run with nobody at the keyboard, as one step of a planned batch
(`.batch/plan.md`). A runner script started you and reads only one thing
when you stop: the status file `.batch/out/<ID>.status`. These rules
override anything that would make you wait for a person.

## Never wait for a person

- Never ask a question and never stop to ask for confirmation: nobody
  answers. `AskUserQuestion` is disabled.
- The owner's answers for this task are in its section of
  `.batch/plan.md` (*Owner decisions*). Follow them. For a small choice
  the plan does not cover, apply the plan's *Open choices* policy: either
  pick the recommended option and record it (a new ADR or a *Log* line
  marked `UNATTENDED CHOICE:` for the owner to review), or block.
- Commands that are not pre-approved are refused automatically. When one
  is refused, use an allowed way to do the same thing (the Read/Edit/Write
  tools instead of `cat`/`sed -i`, a listed `cargo`/`git` form, a script
  already allowed). Never try to get around a deny rule. If no allowed way
  exists, block the task (below) and name the exact command and why it is
  needed, so the owner can allow it.

## Resume, never redo

You may be a fresh session for a task that was already started (after a
crash, a usage limit or a network loss), or a resumed one. Before doing
anything:

1. `git status`, `git branch --show-current`, the task note's status and
   *Log*, and `git log --oneline <base>..<task branch>`.
2. Each workflow step is one commit, so the last commit tells you the last
   finished step. Continue with the next one; never redo or rewrite a
   committed step.
3. Uncommitted changes on the task branch are the unfinished current step:
   review them and finish the step. Changes you cannot explain: keep them
   with `git stash push -m "batch-<ID>-<date>"` (never discard work) and
   note the stash in the task *Log*.

## Spend few tokens

Every tool call re-sends the whole context, so keep it small.

- Filter command output: pipe `cargo test`, `cargo clippy` and
  `scripts/check.sh` through `tail -n 40` (or `grep` for `error|FAILED|
  panicked`), and re-run unfiltered only on a failure you cannot read.
- Read files by range (`offset`/`limit`) or search them; do not read a
  whole large file (`editor.rs`, `app.rs`) for one function.
- Read only the process notes the step needs, once; do not re-read files
  you just wrote or edited.
- Do not explore beyond *Files owned* and the code the task touches; do
  not spawn subagents.
- After an interruption, continue from the git state: do not re-derive
  what the last commit and the task note already say.

## Stay inside the lines

- Follow `implement-task` and `docs/process/Workflow.md` exactly: one
  branch, one commit per step, tests before code, files owned only, no
  shared indexes, no `Co-Authored-By` or agent mentions anywhere.
- Branch from the base branch the prompt names (`main`, or the branch of
  a task this one depends on that is in review). Do not merge it and do
  not push anything unless the plan's *Policies* explicitly say so.
- Never rewrite history, force anything, delete branches, or run commands
  with `sudo`.
- Finish with `scripts/check.sh` green. The runner runs it again on your
  branch and blocks the task if it is red.
- Leave the working tree clean (everything committed, or stashed with a
  note) on every exit path, blocked or not. A dirty tree on another branch
  stops the whole batch.

## When to block

Block instead of guessing when:

- a decision belongs to the owner (architecture, scope, UX) and the plan
  does not answer it and the policy says to block;
- a needed command, tool or file is not allowed or not available;
- you need a file outside *Files owned*;
- the gates stay red after a reasonable effort, or a dependency's work is
  missing or broken;
- a test or result needs a human (a visual check, hardware such as a pen
  tablet, an account).

To block: commit what is done as far as the workflow allows, add a *Log*
line `- <date> — BLOCKED (batch): <what is missing, what you tried, the
recommended fix>` to the task note, commit it as
`docs(<ID>): record blocker for the batch run`, leave the task status as
it is (do not invent a `blocked` status), and write the status file.

Partial work that is red stays committed as the workflow allows (the
`test` step is red by design); do not commit broken code just to save it,
stash it instead.

## Status file: the last thing you do

Write `.batch/out/<ID>.status` with the Write tool:

```
review
<one line: what was done, deviations, anything the owner must check by hand>
```

or

```
blocked
<one line: what blocks it>
<one line: the exact command, permission or decision that would unblock it>
```

Write it only when the task really is in `review` (all steps committed,
`scripts/check.sh` green, task note `status: review`) or really blocked.
If you are interrupted before that, write nothing: the runner resumes you.
