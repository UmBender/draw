#!/usr/bin/env bash
# Unattended runner for a planned batch of tasks (skill: batch-tasks).
#
# Runs one non-interactive Claude Code or agy session per task, one at a
# time, in the order of .batch/queue. Survives what usually kills a night
# run: usage limits (waits for the reset and resumes the same session),
# network loss (waits for the network), crashes and reboots (run again: the
# interrupted task resumes), and commands that were not allowed (the session
# is told to block the task and the batch moves on).
#
# Usage: batch.sh check|run|status|report|reset <ID>|unblock <ID>
#
# Queue lines: `<ID> <base> [agent=claude|agy] [model=<m>] [effort=<e>]`.
# Missing options fall back to the BATCH_* defaults below. Under agy, a
# model whose name already ends in an effort (claude-opus-5-5-high) runs
# as is unless the line sets effort=; `agy models` lists the names.
#
# Environment (all optional):
#   BATCH_TASK_TIMEOUT   seconds per session attempt        (default 14400)
#   BATCH_MAX_ATTEMPTS   failed attempts before giving up   (default 2)
#   BATCH_MAX_DENIALS    agy turns ended by a refused command before the
#                        task counts one failed attempt     (default 6)
#   BATCH_AGENT          default agent: claude or agy       (default claude)
#   BATCH_MODEL          default model     (default sonnet / claude-sonnet-5-5 under agy)
#   BATCH_EFFORT         default effort level               (default medium)
#   BATCH_LIMIT_WAIT     seconds between checks while a usage limit
#                        lasts (no cap: waits until it is restored)
#                                                           (default 900)
#   BATCH_NET_WAIT       seconds between network checks     (default 120)
#   BATCH_STOP_AT        epoch seconds: start nothing new after this
#   BATCH_MIN_FREE_GB    free disk space required           (default 5)
#   CLAUDE_BIN           claude executable                  (default claude)
#   AGY_BIN              agy executable                     (default agy)
set -uo pipefail

ROOT=$(git rev-parse --show-toplevel) || exit 2
cd "$ROOT" || exit 2
SKILL_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
B="$ROOT/.batch"
CLAUDE_BIN=${CLAUDE_BIN:-claude}
AGY_BIN=${AGY_BIN:-agy}
TASK_TIMEOUT=${BATCH_TASK_TIMEOUT:-14400}
MAX_ATTEMPTS=${BATCH_MAX_ATTEMPTS:-2}
MAX_DENIALS=${BATCH_MAX_DENIALS:-6}
DEFAULT_AGENT=${BATCH_AGENT:-claude}
DEFAULT_EFFORT=${BATCH_EFFORT:-medium}
LIMIT_WAIT=${BATCH_LIMIT_WAIT:-900}
NET_WAIT=${BATCH_NET_WAIT:-120}
STOP_AT=${BATCH_STOP_AT:-}
MIN_FREE_GB=${BATCH_MIN_FREE_GB:-5}
EFFORTS='low|medium|high|xhigh|max'

# Agent, model and effort of the task being run (set by use_task).
AGENT=$DEFAULT_AGENT MODEL="" EFFORT=""

log() {
    mkdir -p "$B"
    printf '%s %s\n' "$(date -Is)" "$*" | tee -a "$B/runner.log" >&2
}

die() {
    log "ERROR: $*"
    exit 1
}

# --- per-task state: .batch/state/<ID>, lines key=value ---------------------

get() { # get ID key
    local file="$B/state/$1"
    [[ -f $file ]] || { [[ $2 == status ]] && echo pending; return; }
    local line
    line=$(grep -m1 "^$2=" "$file") || { [[ $2 == status ]] && echo pending; [[ $2 =~ ^(attempts|limit_waits|denials)$ ]] && echo 0; return; }
    printf '%s\n' "${line#*=}"
}

set_state() { # set_state ID key value — atomic rewrite, survives a crash
    local file="$B/state/$1" tmp
    mkdir -p "$B/state"
    tmp=$(mktemp "$B/state/.$1.XXXX")
    { [[ -f $file ]] && grep -v "^$2=" "$file"; printf '%s=%s\n' "$2" "$3"; } >"$tmp"
    mv -f "$tmp" "$file"
}

# --- task notes ----------------------------------------------------------------

task_note() { # path of docs/tasks/<ID> *.md
    local f
    for f in "docs/tasks/$1 "*.md; do [[ -f $f ]] && { printf '%s\n' "$f"; return 0; }; done
    return 1
}

front() { # front ID key — a frontmatter value
    local note
    note=$(task_note "$1") || return 1
    sed -n '/^---$/,/^---$/p' "$note" | grep -m1 "^$2:" | sed "s/^$2:[[:space:]]*//"
}

task_branch() { front "$1" branch; }

task_deps() { # space-separated ids from `depends_on: [T73, T74]`
    front "$1" depends_on | tr -d '[]' | tr ',' ' ' | xargs -n1 2>/dev/null | grep -E '^T[0-9]+' | xargs
}

queue() { # lines "ID BASE" of .batch/queue, comments and blanks skipped
    grep -vE '^[[:space:]]*(#|$)' "$B/queue"
}

in_queue() { queue | awk '{print $1}' | grep -qx "$1"; }

base_branch() { # BASE is `main` or `after:<ID>` (that task's branch)
    case $1 in
        after:*) task_branch "${1#after:}" ;;
        *) printf '%s\n' "$1" ;;
    esac
}

task_opt() { # task_opt ID key: the `key=value` of ID's queue line; fails if unset
    queue | awk -v id="$1" -v k="$2=" '
        $1 == id { for (i = 3; i <= NF; i++) if (index($i, k) == 1) { print substr($i, length(k) + 1); found = 1; exit } }
        END { exit !found }'
}

use_task() { # use_task ID: sets AGENT, MODEL and EFFORT for ID
    AGENT=$(task_opt "$1" agent) || AGENT=$DEFAULT_AGENT
    MODEL=$(task_opt "$1" model) || MODEL=${BATCH_MODEL:-}
    if [[ -z $MODEL ]]; then
        case $AGENT in agy) MODEL=claude-sonnet-5-5 ;; *) MODEL=sonnet ;; esac
    fi
    if EFFORT=$(task_opt "$1" effort); then
        :
    elif [[ $AGENT == agy && $MODEL =~ -($EFFORTS)$ ]]; then
        EFFORT="" # the model name already carries it
    else
        EFFORT=$DEFAULT_EFFORT
    fi
}

agent_bin() { case $AGENT in agy) echo "$AGY_BIN" ;; *) echo "$CLAUDE_BIN" ;; esac; }

agent_desc() { printf '%s %s%s' "$AGENT" "$MODEL" "${EFFORT:+ ($EFFORT)}"; }

# --- environment ---------------------------------------------------------------

net_url() {
    case $AGENT in
        agy) echo "${BATCH_NET_URL:-https://www.googleapis.com}" ;;
        *) echo "${BATCH_NET_URL:-https://api.anthropic.com}" ;;
    esac
}

network_ok() { curl -sS -o /dev/null --max-time 15 "$(net_url)" 2>/dev/null; }

wait_network() {
    network_ok && return 0
    log "network down; waiting"
    until network_ok; do
        past_stop && return 1
        sleep "$NET_WAIT"
    done
    log "network back"
}

past_stop() { [[ -n $STOP_AT ]] && (($(date +%s) >= STOP_AT)); }

free_gb() { df -Pk "$ROOT" | awk 'NR==2 {print int($4 / 1048576)}'; }

tree_clean() { [[ -z $(git status --porcelain --untracked-files=normal) ]]; }

current_branch() { git rev-parse --abbrev-ref HEAD; }

running_on_branch() { # the running task whose branch is checked out, if any
    local id base
    [[ -f $B/queue ]] || return 0
    while read -r id base _; do
        if [[ $(get "$id" status) == running && $(task_branch "$id") == "$(current_branch)" ]]; then
            echo "$id"
            return 0
        fi
    done < <(queue)
}

# --- check ---------------------------------------------------------------------

cmd_check() {
    local ok=0 id base dep tool
    say() { printf '%-6s %s\n' "$1" "$2"; [[ $1 == FAIL ]] && ok=1; return 0; }
    [[ -f $B/plan.md ]] && say ok "plan: $B/plan.md" || say FAIL "no $B/plan.md (run the plan phase first)"
    [[ -f $B/queue ]] && say ok "queue: $(queue | wc -l) task(s)" || { say FAIL "no $B/queue"; return 1; }
    if [[ -f $B/settings.json ]] && jq -e '.permissions.allow | length > 0' "$B/settings.json" >/dev/null 2>&1; then
        say ok "permissions: $(jq '.permissions.allow | length' "$B/settings.json") allowed, $(jq '.permissions.deny // [] | length' "$B/settings.json") denied"
    else
        say FAIL "$B/settings.json missing or has no permissions.allow"
    fi
    local agents agy_models=""
    agents=$(while read -r id base _; do use_task "$id"; echo "$AGENT"; done < <(queue) | sort -u)
    for AGENT in $agents; do
        case $AGENT in
            claude | agy) ;;
            *) say FAIL "unknown agent '$AGENT' (claude or agy)"; continue ;;
        esac
        if command -v "$(agent_bin)" >/dev/null; then
            say ok "$AGENT: $("$(agent_bin)" --version 2>/dev/null | head -1)"
        else
            say FAIL "$AGENT not found ($(agent_bin))"
        fi
        network_ok && say ok "network: $(net_url) reachable" || say FAIL "network: $(net_url) unreachable"
    done
    if grep -qx agy <<<"$agents"; then
        agy_models=$("$AGY_BIN" models 2>/dev/null | awk '{print $1}')
        [[ -n $agy_models ]] || say WARN "agy models: empty list, model names not checked"
        if agy_settings; then
            say ok "agy permissions: $B/agy-settings.json ($(jq '.permissions.allow | length' "$B/agy-settings.json") allowed, $(jq '.permissions.deny | length' "$B/agy-settings.json") denied)"
        else
            say FAIL "cannot write $B/agy-settings.json"
        fi
    fi
    git check-ignore -q "$B/queue" && say ok ".batch/ is git-ignored" || say FAIL ".batch/ is not git-ignored"
    if tree_clean; then
        say ok "working tree clean ($(current_branch))"
    elif [[ -n $(running_on_branch) ]]; then
        say WARN "working tree dirty on $(current_branch): the interrupted task $(running_on_branch) resumes there"
    else
        say FAIL "working tree not clean on $(current_branch)"
    fi
    (($(free_gb) >= MIN_FREE_GB)) && say ok "disk: $(free_gb) GB free" || say FAIL "disk: only $(free_gb) GB free (< $MIN_FREE_GB)"
    command -v systemd-inhibit >/dev/null && say ok "systemd-inhibit: sleep will be blocked" || say WARN "no systemd-inhibit: disable suspend yourself"
    if [[ -f $B/requires ]]; then
        while read -r tool; do
            [[ -z $tool || $tool == \#* ]] && continue
            command -v "$tool" >/dev/null && say ok "tool: $tool" || say FAIL "tool missing: $tool"
        done <"$B/requires"
    fi
    while read -r id base _; do
        task_note "$id" >/dev/null || { say FAIL "$id: no task note"; continue; }
        [[ -n $(task_branch "$id") ]] || say FAIL "$id: task note has no branch"
        [[ -n $(base_branch "$base") ]] || say FAIL "$id: bad base '$base'"
        for dep in $(task_deps "$id"); do
            in_queue "$dep" && continue
            [[ $(front "$dep" status) == done ]] || say WARN "$id: depends on $dep, which is not done and not in the queue"
        done
        use_task "$id"
        if [[ $AGENT == agy && -n $agy_models ]] && ! grep -qx -- "$MODEL${EFFORT:+-$EFFORT}" <<<"$agy_models"; then
            say FAIL "$id: agy has no model '$MODEL'${EFFORT:+ with effort $EFFORT} (see \`agy models\`)"
        fi
        if [[ -n $EFFORT && ! $EFFORT =~ ^($EFFORTS)$ ]]; then
            say FAIL "$id: unknown effort '$EFFORT'"
        fi
        say ok "$id: $(get "$id" status), base $base, $(agent_desc)"
    done < <(queue)
    return $ok
}

# --- one task ------------------------------------------------------------------

ended_in_error() { # ended_in_error RC JSONFILE: non-zero exit, no result or an error result
    (($1 != 0)) || [[ ! -s $2 ]] ||
        jq -e '.is_error == true or (has("status") and .status != "SUCCESS")' "$2" >/dev/null 2>&1
}

rate_limited() { # rate_limited LOGFILE...
    grep -qiE 'usage limit|hit your (usage )?limit|rate.?limit|limit reached|resets (at )?[0-9]|overloaded|API Error: (429|529)|Too Many Requests|quota|RESOURCE_EXHAUSTED|spend.?cap|prepaid|out of credits' "$@" 2>/dev/null
}

network_error() {
    grep -qiE 'ENOTFOUND|ECONNRESET|ECONNREFUSED|ETIMEDOUT|EAI_AGAIN|network error|fetch failed|socket hang up|Connection error|dial tcp|no such host|i/o timeout|connection (reset|refused)|TLS handshake timeout' "$@" 2>/dev/null
}

no_session() { grep -qiE 'no conversation found|(session|conversation) .* not found' "$@" 2>/dev/null; }

effort_args() { [[ -n $EFFORT ]] && printf '%s\n' --effort "$EFFORT"; }

service_ok() { # a one-line probe session of the current agent and model answers
    local out
    local -a effort
    mapfile -t effort < <(effort_args)
    if [[ $AGENT == agy ]]; then
        out=$(timeout 300 "$AGY_BIN" --output-format json --model "$MODEL" "${effort[@]}" \
            --mode accept-edits -p "Reply with the single word OK." </dev/null 2>/dev/null) || return 1
        jq -e '.status == "SUCCESS"' <<<"$out" >/dev/null 2>&1
    else
        out=$(timeout 300 "$CLAUDE_BIN" -p --output-format json --no-session-persistence \
            --model "$MODEL" "${effort[@]}" \
            --permission-mode dontAsk --permission-prompts none \
            "Reply with the single word OK." </dev/null 2>&1) || return 1
        jq -e '.is_error != true' <<<"$out" >/dev/null 2>&1
    fi
}

agy_settings() { # writes .batch/agy-settings.json from .batch/settings.json
    local extra='{}' tmp
    [[ -f $B/agy-extra.json ]] && extra=$(cat "$B/agy-extra.json")
    tmp=$(mktemp "$B/.agy-settings.XXXX")
    jq --arg root "$ROOT" --argjson extra "$extra" \
        --argjson protect "$(jq -n --arg b "$B" --arg s "$SKILL_DIR" '[$b+"/state", $b+"/queue", $b+"/settings.json", $b+"/agy-settings.json", $b+"/agy-extra.json", $s]')" \
        -f "$SKILL_DIR/agy-settings.jq" "$B/settings.json" >"$tmp" 2>/dev/null &&
        mv -f "$tmp" "$B/agy-settings.json" || { rm -f "$tmp"; return 1; }
}

denied_commands() { # denied_commands STREAMFILE: what was refused, on one line
    # A refusal ends the turn, so it is the last tool call; the result names
    # only the kind of action (denied_actions).
    grep -q '"denied_actions":\[{' "$1" 2>/dev/null || return 0
    grep '"step_update"' "$1" |
        jq -r '.step_update | select(.step_type == "tool")
               | .tool_info.parameters.CommandLine // .tool_info.parameters.TargetFile // .tool_name' 2>/dev/null |
        tail -1 | tr '\n\r' '  ' | cut -c1-300
}

wait_until_available() { # wait_until_available LOGFILE...: no cap
    local reset now
    reset=$(grep -ohE 'limit reached\|[0-9]{10}' "$@" 2>/dev/null | head -1 | cut -d'|' -f2)
    now=$(date +%s)
    if [[ -n $reset ]] && ((reset > now)); then
        log "usage limit: waiting until the printed reset, $(date -d "@$reset" -Is)"
        sleep $((reset - now + 60))
    fi
    until service_ok; do
        past_stop && return 1
        log "usage limit (or service unavailable): checking again in ${LIMIT_WAIT}s"
        sleep "$LIMIT_WAIT"
        wait_network || return 1
    done
    log "service available again"
}

prompt_start() { # prompt_start ID BASE
    if [[ $AGENT == agy ]]; then
        # agy loads neither .claude/skills nor CLAUDE.md, nor an extra system
        # prompt: the skill is named by path and the rules go in the prompt.
        printf 'Task %s: follow the implement-task skill (.claude/skills/implement-task/SKILL.md) for %s, as if invoked with `/implement-task %s`.\n\nUnattended batch run (skill batch-tasks). Base branch: %s. Follow the unattended rules below and the plan section for %s in .batch/plan.md. End by writing .batch/out/%s.status.\n\n' "$1" "$1" "$1" "$(base_branch "$2")" "$1" "$1"
        cat "$SKILL_DIR/unattended.md" "$SKILL_DIR/unattended-agy.md"
    else
        printf '/implement-task %s\n\nUnattended batch run (skill batch-tasks). Base branch: %s. Follow the unattended rules in your system prompt and the plan section for %s in .batch/plan.md. End by writing .batch/out/%s.status.\n' "$1" "$(base_branch "$2")" "$1" "$1"
    fi
}

prompt_resume() { # prompt_resume ID BASE REASON
    printf 'You were interrupted (%s). Continue task %s (base branch %s) from where the repository says you are: check git status, the current branch, `git log --oneline %s..HEAD` and the task note before doing anything; never redo a committed step. Then finish and write .batch/out/%s.status.\n' "$3" "$1" "$(base_branch "$2")" "$(base_branch "$2")" "$1"
    if [[ $AGENT == agy ]]; then
        printf '\nThe implement-task skill is .claude/skills/implement-task/SKILL.md. The unattended rules, again:\n\n'
        cat "$SKILL_DIR/unattended.md" "$SKILL_DIR/unattended-agy.md"
    fi
}

session() { # session ID BASE MODE(start|resume) REASON → exit code; output in logs
    local id=$1 base=$2 mode=$3 reason=$4 sid n out err raw prompt
    n=$(($(ls "$B/logs/$id"-*.json 2>/dev/null | wc -l) + 1))
    out="$B/logs/$id-$n.json"
    err="$B/logs/$id-$n.err"
    mkdir -p "$B/logs" "$B/out"
    local -a effort
    mapfile -t effort < <(effort_args)
    if [[ $mode == resume ]]; then
        sid=$(get "$id" session)
        prompt=$(prompt_resume "$id" "$base" "$reason")
        log "$id: session run $n, $(agent_desc), resuming session $sid ($reason)"
    else
        sid=""
        prompt=$(prompt_start "$id" "$base")
        set_state "$id" agent "$AGENT"
    fi

    if [[ $AGENT == agy ]]; then
        # stream-json: the conversation id comes first (init) and the result
        # last, also when a refused command ends the turn with no output.
        raw="$B/logs/$id-$n.jsonl"
        agy_settings || log "$id: could not refresh $B/agy-settings.json; using the last one"
        [[ $mode == start ]] && log "$id: session run $n, $(agent_desc), new conversation"
        local -a args=(--output-format stream-json --model "$MODEL" "${effort[@]}"
            --mode accept-edits --settings "$B/agy-settings.json")
        [[ -n $sid ]] && args+=(--conversation "$sid")
        timeout --kill-after=60 "$TASK_TIMEOUT" "$AGY_BIN" "${args[@]}" -p "$prompt" \
            >"$raw" 2>"$err" </dev/null
        local rc=$?
        grep '"event":"result"' "$raw" 2>/dev/null | tail -1 | jq '.result' >"$out" 2>/dev/null
        sid=$(grep -m1 '"event":"init"' "$raw" 2>/dev/null | jq -r '.conversation_id // empty' 2>/dev/null)
        [[ -n $sid ]] && set_state "$id" session "$sid"
        set_state "$id" denied "$(denied_commands "$raw")"
    else
        local -a common=(
            -p --output-format json
            --model "$MODEL" "${effort[@]}"
            --permission-mode dontAsk --permission-prompts none
            --settings "$B/settings.json"
            --disallowedTools AskUserQuestion
            --append-system-prompt "$(cat "$SKILL_DIR/unattended.md")"
        )
        if [[ -n $sid ]]; then
            common+=(--resume "$sid")
        else
            sid=$(uuidgen)
            set_state "$id" session "$sid"
            log "$id: session run $n, $(agent_desc), new session $sid"
            common+=(--session-id "$sid")
        fi
        timeout --kill-after=60 "$TASK_TIMEOUT" "$CLAUDE_BIN" "${common[@]}" "$prompt" \
            >"$out" 2>"$err" </dev/null
        local rc=$?
    fi
    echo "$out $err $rc"
}

verify_review() { # the session says review: the branch must exist and be green
    local id=$1 branch prev
    branch=$(task_branch "$id")
    git rev-parse --verify -q "$branch" >/dev/null || { echo "branch $branch does not exist"; return 1; }
    tree_clean || { echo "working tree left dirty"; return 1; }
    prev=$(current_branch)
    git switch -q "$branch" || { echo "cannot switch to $branch"; return 1; }
    if ! scripts/check.sh >"$B/logs/$id-verify.log" 2>&1; then
        git switch -q "$prev"
        echo "scripts/check.sh red on $branch (see .batch/logs/$id-verify.log)"
        return 1
    fi
    git switch -q "$prev"
}

run_task() { # run_task ID BASE → 0 done (any final status), 1 stop the batch
    local id=$1 base=$2 mode=start reason="" result out err rc status note attempts denials refused
    use_task "$id"
    [[ $(get "$id" status) == running && -n $(get "$id" session) ]] && { mode=resume; reason="runner restarted"; }
    while :; do
        # A session belongs to one agent: after the queue line changed agent,
        # a new session continues from the repository state.
        if [[ $mode == resume && $(get "$id" agent) != "$AGENT" ]]; then
            log "$id: agent is now $AGENT (session was $(get "$id" agent)); starting a new session"
            mode=start
        fi
        past_stop && { log "stop time reached; $id stays $(get "$id" status)"; return 1; }
        wait_network || return 1
        (($(free_gb) >= MIN_FREE_GB)) || { log "disk below ${MIN_FREE_GB} GB; stopping"; return 1; }
        if ! tree_clean && [[ $(current_branch) != "$(task_branch "$id")" ]]; then
            log "working tree dirty on $(current_branch), not $id's branch; stopping the batch"
            return 1
        fi
        attempts=$(get "$id" attempts)
        if ((attempts >= MAX_ATTEMPTS)); then
            set_state "$id" status failed
            set_state "$id" note "gave up after $attempts failed attempts (see .batch/logs/$id-*)"
            log "$id: failed after $attempts attempts"
            return 0
        fi
        rm -f "$B/out/$id.status"
        set_state "$id" status running
        set_state "$id" started "$(date -Is)"
        read -r out err rc < <(session "$id" "$base" "$mode" "$reason")

        if [[ -f $B/out/$id.status ]]; then
            status=$(head -1 "$B/out/$id.status" | tr -d '[:space:]')
            note=$(sed -n '2,$p' "$B/out/$id.status" | tr '\n' ' ' | cut -c1-400)
            case $status in
                review)
                    if result=$(verify_review "$id"); then
                        set_state "$id" status review
                    else
                        status=blocked
                        note="session said review, but $result"
                        set_state "$id" status blocked
                    fi
                    ;;
                blocked) set_state "$id" status blocked ;;
                *)
                    status=blocked
                    note="unknown status '$status' in .batch/out/$id.status: $note"
                    set_state "$id" status blocked
                    ;;
            esac
            set_state "$id" note "$note"
            set_state "$id" finished "$(date -Is)"
            log "$id: $status — $note"
            return 0
        fi

        if no_session "$out" "$err"; then
            log "$id: session not found; starting a new one from the repository state"
            mode=start
            continue
        fi
        if { ended_in_error "$rc" "$out" && network_error "$out" "$err"; } || ! network_ok; then
            wait_network || return 1
            mode=resume reason="network loss"
            continue
        fi
        # A usage limit, recognised by its message or not: the probe fails
        # while it lasts. Wait until it is restored; not a failed attempt.
        if ended_in_error "$rc" "$out" && { rate_limited "$out" "$err" || ! service_ok; }; then
            set_state "$id" limit_waits $(($(get "$id" limit_waits) + 1))
            wait_until_available "$out" "$err" || return 1
            mode=resume reason="usage limit"
            continue
        fi
        # agy ends the turn at a refused command; resume and tell the
        # session, up to MAX_DENIALS times before it costs an attempt.
        refused=$(get "$id" denied)
        if [[ $AGENT == agy ]] && { [[ -n $refused ]] || jq -e '(.denied_actions // []) | length > 0' "$out" >/dev/null 2>&1; }; then
            denials=$(($(get "$id" denials) + 1))
            set_state "$id" denials "$denials"
            if ((denials <= MAX_DENIALS)); then
                log "$id: a command was refused (${refused:-unknown}); resuming ($denials/$MAX_DENIALS)"
                mode=resume
                reason="a command was refused by the permission rules: ${refused:-unknown}. Do not retry it or a rephrased form; use an allowed alternative, or block the task naming that command"
                continue
            fi
            set_state "$id" denials 0
        fi
        set_state "$id" attempts $((attempts + 1))
        if ((rc == 124 || rc == 137)); then
            reason="the attempt timed out after ${TASK_TIMEOUT}s"
        else
            reason="the session ended (exit $rc) without writing the status file"
        fi
        log "$id: $reason"
        mode=resume
    done
}

skip_reason() { # skip_reason ID BASE: why ID cannot run now, if a queued
    # dependency or the task it branches from (`after:<ID>`) did not finish
    local dep st after=""
    [[ ${2:-} == after:* ]] && after=${2#after:}
    for dep in $(task_deps "$1") $after; do
        in_queue "$dep" || continue
        st=$(get "$dep" status)
        case $st in
            review | done) ;;
            blocked | failed | skipped) echo "dependency $dep is $st"; return 0 ;;
            *) echo "dependency $dep is still $st (queue order?)"; return 0 ;;
        esac
    done
    return 1
}

cmd_run() {
    mkdir -p "$B"
    if [[ -z ${BATCH_INHIBITED:-} ]] && command -v systemd-inhibit >/dev/null; then
        export BATCH_INHIBITED=1
        exec systemd-inhibit --what=sleep:idle:handle-lid-switch --who=batch-tasks \
            --why="unattended task batch" "$0" run
    fi
    exec 9>"$B/lock"
    flock -n 9 || die "another runner holds $B/lock"
    cmd_check >"$B/check.log" 2>&1 || die "pre-flight check failed (see $B/check.log)"
    log "batch started (pid $$)"
    local id base why
    while read -r id base _; do
        case $(get "$id" status) in review | blocked | failed | skipped | done) continue ;; esac
        if why=$(skip_reason "$id" "$base"); then
            set_state "$id" status skipped
            set_state "$id" note "$why"
            log "$id: skipped — $why"
            continue
        fi
        run_task "$id" "$base" || { log "batch stopped"; cmd_report >/dev/null; exit 1; }
        if tree_clean && [[ $(current_branch) != main ]]; then git switch -q main; fi
    done < <(queue)
    log "batch finished"
    cmd_report
}

# --- status, report, reset -------------------------------------------------------

cmd_status() {
    local id base
    printf '%-5s %-9s %-8s %s\n' TASK STATUS ATTEMPTS NOTE
    while read -r id base _; do
        printf '%-5s %-9s %-8s %s\n' "$id" "$(get "$id" status)" "$(get "$id" attempts)" "$(get "$id" note | cut -c1-100)"
    done < <(queue)
    [[ -f $B/lock ]] && flock -n "$B/lock" true 2>/dev/null || echo "(a runner is active)"
}

cmd_report() {
    local id base branch from report="$B/report.md"
    {
        echo "# Batch report — $(date -Is)"
        echo
        while read -r id base _; do
            branch=$(task_branch "$id")
            from=$(base_branch "$base")
            echo "## $id — $(get "$id" status)"
            echo
            echo "- branch: \`$branch\` (base \`$from\`), attempts: $(get "$id" attempts), usage-limit waits: $(get "$id" limit_waits)"
            use_task "$id"
            echo "- agent: $(agent_desc)"
            [[ -n $(get "$id" note) ]] && echo "- note: $(get "$id" note)"
            echo "- logs: \`.batch/logs/$id-*\`"
            if git rev-parse --verify -q "$branch" >/dev/null; then
                echo
                echo '```'
                git log --oneline "$from..$branch" 2>/dev/null
                echo '```'
            fi
            echo
        done < <(queue)
        echo "## Runner log (last 30 lines)"
        echo
        echo '```'
        tail -30 "$B/runner.log" 2>/dev/null
        echo '```'
    } >"$report"
    cat "$report"
}

cmd_reset() { # forget everything about a task: it runs from scratch
    [[ -n ${1:-} ]] || die "usage: batch.sh reset <ID>"
    rm -f "$B/state/$1" "$B/out/$1.status"
    log "$1: reset to pending"
}

cmd_unblock() { # run again, resuming its last session
    [[ -n ${1:-} ]] || die "usage: batch.sh unblock <ID>"
    set_state "$1" status running
    set_state "$1" attempts 0
    set_state "$1" denials 0
    set_state "$1" denied ""
    rm -f "$B/out/$1.status"
    log "$1: unblocked; it resumes its last session on the next run"
}

# Sourced (tests): define the functions only.
[[ ${BASH_SOURCE[0]} == "$0" ]] || return 0

case ${1:-} in
    check) cmd_check ;;
    run) cmd_run ;;
    status) cmd_status ;;
    report) cmd_report ;;
    reset) cmd_reset "${2:-}" ;;
    unblock) cmd_unblock "${2:-}" ;;
    *) sed -n '2,/^set -uo/{/^#/p}' "$0"; exit 2 ;;
esac
