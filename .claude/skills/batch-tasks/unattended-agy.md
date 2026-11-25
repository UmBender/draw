## Running under the Antigravity CLI (`agy`)

This session runs in `agy`, not Claude Code. The rules above still apply,
with these differences:

- Nothing from `.claude/` or `CLAUDE.md` was loaded for you. Read
  `CLAUDE.md` first. A skill named in these rules or in a skill you follow
  (`implement-task`, `rust-skills`, `batch-tasks`) is the file
  `.claude/skills/<name>/SKILL.md`; read it and follow it as if it had been
  invoked. A slash command such as `/implement-task T04` means "follow
  that skill for T04".
- Tool names: "the Read/Edit/Write tools" are your file viewing and
  editing tools; `AskUserQuestion` is your `ask_question` tool. Never call
  `ask_question`, `ask_permission` or `ask_custom_permission`: nobody
  answers. Never set `BypassSandbox`.
- Command permissions match the command line word by word as a prefix of
  an allow rule. Anything else in the line disables the match and the
  whole line is refused: no `$(…)`, `${…}`, backticks, `{a,b}`, `;`, `&&`,
  `||`, `echo` added after the command, or `git -c …`. A plain `| tail -n
  40` or `| grep …` is fine when both sides are allowed. Run the command
  exactly as you need it, nothing appended.
- A refused command ends your turn at once. The runner resumes you and
  says that a command was refused: do not retry the same command or a
  rephrased one; use an allowed alternative, or block the task naming the
  command and why it is needed.
- A command you start in the background still has to finish before you
  stop: wait for its result, never end the turn while `cargo` or
  `scripts/check.sh` is running.
