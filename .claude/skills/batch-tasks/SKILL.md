---
name: batch-tasks
description: Plan a list of tasks from docs/tasks with the owner, then run them unattended (e.g. overnight), one Claude Code or Antigravity (agy) session at a time, each task with its own agent, model and effort, surviving usage limits, network loss, crashes, reboots and commands that were not pre-approved; blocked tasks are recorded and skipped so the rest continues. Use when asked to plan, run, resume or report a batch of tasks, or to "implement these tasks while I sleep".
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
session's system prompt), `unattended-agy.md` (extra rules for agy
sessions), `agy-settings.jq` (turns the Claude permission rules into agy
rules), `settings.template.json` (base allow/deny list),
`plan.template.md`. State lives in `.batch/` (git-ignored).

## Agent, model and effort per task

Each `.batch/queue` line is `<ID> <base> [agent=claude|agy] [model=<m>]
[effort=<e>]`; what a line leaves out comes from `BATCH_AGENT` (claude),
`BATCH_MODEL` (sonnet, or claude-sonnet-5-5 under agy) and `BATCH_EFFORT`
(medium). `batch.sh check` prints each task's choice and fails on an agy
model/effort pair that `agy models` does not list.

```
T75 main agent=claude model=opus effort=high      # design-heavy
T54 after:T75 model=sonnet effort=medium           # routine
T80 main agent=agy model=gemini-4-neon effort=high
T81 main agent=agy model=claude-opus-5-5-high      # effort in the name
```

Rough guide: `opus`/`high` for tasks with an architecture choice, unsafe
code or a tricky algorithm; `sonnet`/`medium` for ordinary features;
`haiku`/`low` (or a flash model) only for docs and mechanical edits. Under
agy, `effort=` must be one the model offers (Claude models: low, medium,
high), and a model name ending in an effort runs as is. A session belongs
to one agent: changing a task's agent starts a new session from the git
state instead of resuming.

### What differs under agy

Tested with agy 1.5.16:

- **Permissions:** `.batch/agy-settings.json` is regenerated from
  `.batch/settings.json` before every agy session: `Bash(cmd *)` becomes
  `command(cmd)` (word prefix), a pattern with other `*` becomes
  `command(regex:…)`, file tools become `write_file(<repo>)` and
  `write_file(<repo>/.git)`, and writing to `.batch/state`, `.batch/queue`,
  the settings and this skill is denied. Rules that exist only for agy go in
  `.batch/agy-extra.json` (`{"permissions":{"allow":[…],"deny":[…]}}`),
  merged in.
- **Mode `accept-edits`**, never `auto`: in `-p`, `auto` lets the agent
  approve its own `BypassSandbox` (network, writes outside the repo)
  without asking. Under `accept-edits` a command outside the allow list is
  refused and **ends the turn**; the runner resumes the conversation naming
  the refused command, up to `BATCH_MAX_DENIALS` (6) times before it
  counts as a failed attempt.
- **No CLAUDE.md, `.claude/skills` or extra system prompt** is loaded:
  the prompt names `.claude/skills/implement-task/SKILL.md` and carries
  `unattended.md` + `unattended-agy.md`.
- **Output:** `stream-json`. The raw stream is `.batch/logs/<ID>-<n>.jsonl`;
  the result object goes to `<ID>-<n>.json`. The conversation id comes from
  the stream, since agy cannot be given one.
- **Usage limits:** agy prints no reset time. The runner recognises
  quota/429 messages, or a probe with the task's model that fails, and
  waits as for Claude.

## What can go wrong, and what happens

| Event | Handling |
|-------|----------|
| Usage limit | Waits until the limit is restored, with no cap: sleeps until the printed reset time if there is one, then checks every `BATCH_LIMIT_WAIT` (15 min) with a one-line probe session until Claude answers again, and resumes the same session (`--resume`). The probe also catches a limit whose message is not recognised. Never counted as a failed attempt. |
| Network down | Runner checks `api.anthropic.com` (agy: `www.googleapis.com`) before every attempt and after a failure, waits until it answers, resumes. |
| PC crash, reboot, killed runner | State files are written atomically. Run `batch.sh run` again: the task marked `running` resumes its saved session, or a new session continues from the git state (every workflow step is a commit). |
| Laptop going to sleep | The runner holds a `systemd-inhibit` lock (sleep, idle, lid switch) while it runs. |
| Command not pre-approved | Refused automatically (`--permission-mode dontAsk --permission-prompts none`; agy: `--mode accept-edits` in `-p`), never a prompt. The session uses an allowed alternative or blocks the task, naming the command to allow. Under agy the refusal ends the turn and the runner resumes it. |
| A question for the owner | `AskUserQuestion` is disabled. The session follows the plan's answers and its *Open choices* policy, or blocks. |
| Session hangs or loops | Each attempt is killed after `BATCH_TASK_TIMEOUT` (4 h) and resumed; after 2 failed attempts the task is `failed`. |
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
6. **Agent, model, effort.** Propose one per task (guide above) from its
   size, risk and kind of work, and confirm with the owner in the same
   `AskUserQuestion` batch. Write them on the queue lines; leave out what
   matches the defaults.
7. **Risks.** Mark per task what is likely to block (a visual check,
   hardware, a missing package, a big design choice) and decide with the
   owner: prepare it now, accept the risk, or drop the task.
8. **Write the files:**
   - `.batch/plan.md` from `plan.template.md` (policies, prerequisites,
     order, one section per task with owner decisions, defaults, extra
     commands, block conditions, manual checks);
   - `.batch/queue`: one line per task, `<ID> <base> [agent=…] [model=…]
     [effort=…]`, in order;
   - `.batch/settings.json`: `settings.template.json` plus the extra
     allow rules (narrow patterns, never `Bash(*)`); add `git merge` /
     `git push` rules only if the policies allow them, and remove the
     matching deny rules;
   - `.batch/requires`: one tool per line;
   - `.batch/agy-extra.json`, only when an agy task needs rules the
     translation cannot express.
9. **Show** the owner the plan, the allow additions and the deny list, and
   run `batch.sh check`; fix every `FAIL`.
10. **Hand over** the start command (below). Do not start it yourself: the
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
  result (agy: also the `.jsonl` stream and `denied=` in the state file), explain the blocker, and propose the fix (allow a command,
  answer a decision, install a tool). Then `batch.sh unblock <ID>`
  (resumes the last session) or `batch.sh reset <ID>` (starts over), and
  run again. For `review` tasks, list the manual checks and
  `UNATTENDED CHOICE:` lines for the owner, then integrate as usual
  (merge `--no-ff` in queue order, shared indexes, CHANGELOG).
- `resume`: same as starting the run again.
