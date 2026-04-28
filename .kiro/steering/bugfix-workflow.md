---
inclusion: manual
---

# Bugfix Workflow

Use this workflow for fixing bugs.

## Steps

1. **Reproduce** — understand exact conditions triggering the bug (reproduction steps)
2. **Find root cause** — use `context-gatherer` sub-agent to locate relevant files
3. **Write failing test** — property test or integration test reproducing the bug
4. **Fix** — minimal change, no refactoring
5. **Verify** — test passes, clippy clean, other tests not broken
6. **Update CHANGELOG.md** — `### Fixed` section with bug description and issue link

## When to Use Bugfix Spec

- Bug in critical path (auth, credentials, protocol handshake)
- Previous fix attempts caused regressions
- Root cause not obvious
- Documentation needed for the team

## When a Quick Fix in Chat is Enough

- Typo, simple logic error
- One-line change with obvious root cause
