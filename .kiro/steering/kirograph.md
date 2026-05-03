---
inclusion: auto
---

# KiroGraph — Semantic Code Graph

When `.kirograph/` exists in the workspace root, prefer KiroGraph MCP tools over file scanning for code exploration:

## Tool Priority

1. **Understanding a task** → `kirograph_context` (single call replaces multiple file reads)
2. **Finding a symbol** → `kirograph_search` (faster than grepSearch for symbol names)
3. **Who calls this?** → `kirograph_callers`
4. **What does this call?** → `kirograph_callees`
5. **Impact of a change** → `kirograph_impact` (before modifying any symbol)
6. **Type hierarchy** → `kirograph_type_hierarchy`
7. **Connection between symbols** → `kirograph_path`
8. **Dead code** → `kirograph_dead_code`
9. **Circular deps** → `kirograph_circular_deps`
10. **Architecture overview** → `kirograph_architecture`

## When to Still Use File Tools

- Reading actual file content for editing (kirograph gives locations, not full files)
- Writing/modifying code
- Running commands
- Files not yet indexed (just created, not synced)

## RustConn-Specific Notes

- Project has 3 crates: `rustconn-core`, `rustconn-cli`, `rustconn`
- Use `kirograph_coupling` to verify crate boundary health
- Use `kirograph_impact` before cross-crate refactors
- Index covers only `*.rs` files (configured in `.kirograph/config.json`)
