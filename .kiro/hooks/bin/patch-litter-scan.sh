#!/usr/bin/env bash
# Stop-hook producer: flag reject/backup litter left by a failed patch apply.
#
# The class this catches. A delegated agent (rust-quality-check on 2026-09-22)
# tried to apply a patch to rustconn/src/embedded_vnc_types.rs, it applied only
# partially, and it left the tree with an `impl` nested inside another `impl` —
# a parse error — plus `.orig` and `.rej` files. The main agent did not notice
# until scripts/verify.sh failed several minutes later on a run it had launched
# against the broken tree. A `*.rej` or `*.orig` in the tree is the unambiguous
# fingerprint of that failure, and nothing in this repo legitimately produces
# one: patch/git-apply reject files and editor/patch backups are never checked
# in and never wanted.
#
# Why only reject/backup litter, and not "files changed but not in the edit
# journal". That second signal is tempting — a sub-agent's edits may bypass the
# main session's PostToolUse journal — but it is exactly the false-positive that
# cost five wasted agent loops on 2026-09-06 (see bin/edit-journal.sh): in a
# checkout shared with the IDE or a second session, "dirty but not ours" is the
# normal state, not a fault. A `.rej`/`.orig` file has no such innocent reading,
# so this scans for that alone and stays silent otherwise.
#
# Delivery. This is a `command` action on Stop, whose stdout goes nowhere, so it
# appends to the shared report channel (target/.kiro-session-report) that
# bin/session-report.sh's `flush` prints on the next UserPromptSubmit and then
# deletes. The channel contract lives in session-report.sh: producers APPEND a
# self-contained paragraph, flush is the single consumer. This is the third
# producer (after the debug-leftover scan and flatpak-manifest-check), and per
# that contract it needs no change to flush — just this append.
#
# Fails OPEN and silent: a broken litter scan must never interrupt or delay a
# session, and a clean tree must add zero tokens.

set -uo pipefail

trap 'exit 0' ERR

repo=$(git rev-parse --show-toplevel 2>/dev/null) || exit 0
cd "$repo" 2>/dev/null || exit 0

report="target/.kiro-session-report"

# Find reject/backup litter anywhere in the working tree, tracked or not, but
# never inside target/ (our own session state) or .git/. Prefer `git ls-files`
# so .gitignore and the repo boundary are respected; -o --exclude-standard
# catches the untracked ones, --cached the improbable case someone staged one.
litter=$(
    {
        git ls-files -o --exclude-standard -- '*.rej' '*.orig' 2>/dev/null || true
        git ls-files --cached -- '*.rej' '*.orig' 2>/dev/null || true
    } | grep -vE '^target/' | sort -u || true
)

[ -n "$litter" ] || exit 0

mkdir -p target 2>/dev/null || exit 0

# Append, per the shared-channel contract. Re-appending on the next Stop is
# intended: flush consumed the previous copy, and the litter is still on disk
# until someone removes it.
{
    printf 'PATCH LITTER: reject/backup files in the tree, the fingerprint of a partially applied patch:\n'
    printf '%s\n' "$litter" | sed 's/^/  /'
    printf 'A `.rej`/`.orig` here usually means a delegated agent applied a patch that only partly took.\n'
    printf 'Inspect the named file(s) for corruption (e.g. a broken merge or a misplaced block), fix the\n'
    printf 'source file, then delete the .rej/.orig. Do not commit them.\n'
} >>"$report" 2>/dev/null || true

exit 0
