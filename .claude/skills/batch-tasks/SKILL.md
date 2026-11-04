---
name: batch-tasks
description: Plan a list of tasks from docs/tasks with the owner, then run them unattended (e.g. overnight), one Claude Code session at a time, surviving usage limits, network loss, crashes, reboots and commands that were not pre-approved; blocked tasks are recorded and skipped so the rest continues. Use when asked to plan, run, resume or report a batch of tasks, or to "implement these tasks while I sleep".
argument-hint: plan <IDs…> | status | report | resume | triage
---

# batch-tasks

Implements several tasks without anyone at the keyboard. Two phases:

1. **Plan** (interactive, `/batch-tasks plan T75 T54 …`): settle everything
   that would need the owner, and pre-approve every command, *before* the
   run. Writes `.batch/plan.md`, `.batch/queue`, `.batch/settings.json` and
   `.batch/requires`.
2. **Run** (unattended, started by the owner from a terminal):
   `.claude/skills/batch-tasks/batch.sh run`. One non-interactive session
   per task, in queue order, never two at once. Each session follows
   `implement-task` plus `unattended.md`.

Files: `batch.sh` (runner), `unattended.md` (rules appended to every
session's system prompt), `settings.template.json` (base allow/deny
list), `plan.template.md`. State lives in `.batch/` (git-ignored).

## What can go wrong, and what happens

| Event | Handling |
|-------|----------|
| Usage limit | Waits until the limit is restored, with no cap: sleeps until the printed reset time if there is one, then checks every `BATCH_LIMIT_WAIT` (15 min) with a one-line probe session until Claude answers again, and resumes the same session (`--resume`). The probe also catches a limit whose message is not recognised. Never counted as a failed attempt. |
| Network down | Runner checks `api.anthropic.com` before every attempt and after a failure, waits until it answers, resumes. |
| PC crash, reboot, killed runner | State files are written atomically. Run `batch.sh run` again: the task marked `running` resumes its saved session, or a new session continues from the git state (every workflow step is a commit). |
| Laptop going to sleep | The runner holds a `systemd-inhibit` lock (sleep, idle, lid switch) while it runs. |
| Command not pre-approved | Refused automatically (`--permission-mode dontAsk --permission-prompts none`), never a prompt. The session uses an allowed alternative or blocks the task, naming the command to allow. |
| A question for the owner | `AskUserQuestion` is disabled. The session follows the plan's answers and its *Open choices* policy, or blocks. |
| Session hangs or loops | Each attempt is killed after `BATCH_TASK_TIMEOUT` (4 h) and resumed; after 3 failed attempts the task is `failed`. |
| Session claims success wrongly | Runner re-runs `scripts/check.sh` on the task branch; red → `blocked`. |
| Blocked or failed task | Recorded with the reason; the batch moves on. Queued tasks that depend on it become `skipped`. |
| Two runners | `flock` on `.batch/lock`: the second one exits. |
| Dirty tree on a foreign branch, disk below 5 GB | The batch stops (safe) instead of mixing work. |

Task statuses in `.batch/state/<ID>`: `pending`, `running`, `review`,
`blocked`, `failed`, `skipped`. Task notes keep the workflow statuses; a
blocker is a *Log* line, never a new status.

## Plan phase (`plan <IDs…>`)

Do this with the owner present; it is the only time questions are allowed.

1. **Preflight.** Working tree clean on `main`; `.batch/` git-ignored (add
   `/.batch/` to `.gitignore` if missing, with the owner's OK). If an old
   `.batch/` exists, show `batch.sh status` and ask whether to archive it
   (`mv .batch .batch-<date>`).
2. **Read** every listed task note, its `depends_on`, *Files owned*,
   *Proposal* and expected ACs, plus the ADRs and code it touches.
3. **Order.** Topological order by `depends_on`. A dependency outside the
   batch must be `done`; otherwise ask whether to add it or drop the task.
   Base of each task: `main`, or `after:<ID>` when it depends on a batch
   task (merge policy "no"). Prefer independent tasks early so a blocker
   costs less.
4. **Questions.** For each task, list every decision the spec step would
   otherwise ask the owner (architecture choices, scope, tools, risky
   deviations, out-of-scope boundaries) and ask them now, batched with
   `AskUserQuestion` (recommended option first). Also ask the policies:
   merge (default no), push (default no), open choices
   (`decide-and-record` or `block`), stop time.
5. **Commands.** For each task, list the commands its steps will need
   beyond `settings.template.json` (tools, scripts, test runners,
   compositors…). Check each with `command -v`; anything missing becomes a
   prerequisite (e.g. a `sudo dnf install …` line the owner runs, since
   `sudo` is denied). Put tool names in `.batch/requires`.
6. **Risks.** Mark per task what is likely to block (a visual check,
   hardware, a missing package, a big design choice) and decide with the
   owner: prepare it now, accept the risk, or drop the task.
7. **Write the files:**
   - `.batch/plan.md` from `plan.template.md` (policies, prerequisites,
     order, one section per task with owner decisions, defaults, extra
     commands, block conditions, manual checks);
   - `.batch/queue`: one line per task, `<ID> <base>`, in order;
   - `.batch/settings.json`: `settings.template.json` plus the extra
     allow rules (narrow patterns, never `Bash(*)`); add `git merge` /
     `git push` rules only if the policies allow them, and remove the
     matching deny rules;
   - `.batch/requires`: one tool per line.
8. **Show** the owner the plan, the allow additions and the deny list, and
   run `batch.sh check`; fix every `FAIL`.
9. **Hand over** the start command (below). Do not start it yourself: the
   runner must outlive this session, and only one session may run.

## Starting the run (owner, in a terminal)

```sh
.claude/skills/batch-tasks/batch.sh check       # all ok?
tmux new -d -s batch '.claude/skills/batch-tasks/batch.sh run'
# or: nohup .claude/skills/batch-tasks/batch.sh run >/dev/null 2>&1 &
```

Close interactive Claude sessions on this repo first (one session at a
time). Optional limits: `BATCH_STOP_AT=$(date -d 'tomorrow 07:30' +%s)`,
`BATCH_TASK_TIMEOUT=10800`.

After a crash or reboot: run the same command again. To resume
automatically after a reboot, a user service can start it at boot
(needs `loginctl enable-linger $USER`):

```ini
# ~/.config/systemd/user/draw-batch.service
[Unit]
Description=draw task batch
After=network-online.target

[Service]
Type=simple
WorkingDirectory=%h/repos/draw
ExecStart=%h/repos/draw/.claude/skills/batch-tasks/batch.sh run

[Install]
WantedBy=default.target
```

Enable it only for the night (`systemctl --user enable --now
draw-batch`), disable it in the morning; a finished batch exits at once.

## Morning (`status`, `report`, `triage`)

- `batch.sh status` — one line per task; `batch.sh report` — writes and
  prints `.batch/report.md` (status, note, commits, log paths).
- **Triage** (`/batch-tasks triage`): for each `blocked`/`failed` task,
  read its status file, *Log* and the last `.batch/logs/<ID>-*.json`
  result, explain the blocker, and propose the fix (allow a command,
  answer a decision, install a tool). Then `batch.sh unblock <ID>`
  (resumes the last session) or `batch.sh reset <ID>` (starts over), and
  run again. For `review` tasks, list the manual checks and
  `UNATTENDED CHOICE:` lines for the owner, then integrate as usual
  (merge `--no-ff` in queue order, shared indexes, CHANGELOG).
- `resume`: same as starting the run again.
