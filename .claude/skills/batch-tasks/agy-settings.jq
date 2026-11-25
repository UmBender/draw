# Translates .batch/settings.json (Claude Code rules) into agy permission rules.
# Args: $root (repo), $protect (paths no session may write), $extra (agy rules
# from .batch/agy-extra.json, merged in). Used by batch.sh.
# Bash(<pattern>) → command(<prefix>) or command(regex:<tokens>)
def rule:
  capture("^Bash\\((?<c>.*)\\)$").c
  | sub(" \\*$"; "")
  | if test("\\*") then
      "command(regex:" + (split(" ") | map(gsub("(?<m>[.+?^$(){}|\\[\\]\\\\])"; "\\\(.m)") | gsub("\\*"; ".*")) | join(" ")) + ")"
    else "command(" + . + ")" end;
def rules($k): [(.permissions[$k] // [])[] | select(startswith("Bash(")) | rule] | unique;
{
  permissions: {
    allow: (rules("allow") + ["write_file(\($root))", "write_file(\($root)/.git)"] + ($extra.permissions.allow // [])),
    deny: (rules("deny") + ($protect | map("write_file(\(.))")) + ($extra.permissions.deny // []))
  }
}
