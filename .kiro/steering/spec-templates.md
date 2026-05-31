---
inclusion: manual
---

# Spec Templates — RustConn

Templates for quickly creating specs of various types.

## Design-First: New Protocol

Use when the architecture is already known (Protocol trait, dialog, CLI handler).
Skips requirements, goes straight to design → tasks.

### design.md template

```markdown
# Design: {Protocol} Protocol Support

## Architecture

### rustconn-core changes
- `rustconn-core/src/protocol/{protocol}.rs` — implement `Protocol` trait
- `ProtocolType::{Protocol}` variant in enum
- Capabilities: has_terminal, has_password, has_port, has_username
- Default port: {port}
- Connection logic in `connect()` method

### rustconn changes
- `rustconn/src/dialogs/connection/{protocol}_tab.rs` — connection dialog tab
- Session handling in `rustconn/src/session/`
- Sidebar icon mapping

### rustconn-cli changes
- `rustconn-cli/src/commands/{protocol}.rs` — CLI connect command

### Data model
- Fields in Connection struct (via ProtocolConfig enum or dedicated struct)
- Serialization compatibility (serde skip_serializing_if)

## Dependencies
- New crates needed: {list or "none"}
- Feature flags: {if optional}
```

### tasks.md template

```markdown
# Tasks: {Protocol} Protocol

## Task 1: Core protocol implementation (rustconn-core)
- [ ] 1.1 Add `ProtocolType::{Protocol}` variant to enum
- [ ] 1.2 Create `rustconn-core/src/protocol/{protocol}.rs`
- [ ] 1.3 Implement `Protocol` trait (capabilities, default_port, connect)
- [ ] 1.4 Register module in `protocol/mod.rs`
- [ ] 1.5 Add protocol-specific fields to Connection model (if needed)

## Task 2: Property tests (rustconn-core)
- [ ] 2.1 Protocol type serialization round-trip
- [ ] 2.2 Capabilities correctness
- [ ] 2.3 Connection validation (port range, required fields)

## Task 3: Connection dialog (rustconn)
- [ ] 3.1 Create `rustconn/src/dialogs/connection/{protocol}_tab.rs`
- [ ] 3.2 Add tab to connection dialog notebook
- [ ] 3.3 Wire save/load for protocol-specific fields
- [ ] 3.4 All labels via `i18n()`

## Task 4: Session handling (rustconn)
- [ ] 4.1 Handle connection in session manager
- [ ] 4.2 Tab creation (terminal or embedded widget)
- [ ] 4.3 Disconnect/reconnect logic

## Task 5: CLI handler (rustconn-cli)
- [ ] 5.1 Add subcommand to CLI
- [ ] 5.2 Implement `cmd_{protocol}()` using core connect logic

## Task 6: i18n & accessibility
- [ ] 6.1 Wrap all strings in `i18n()`
- [ ] 6.2 Run `po/update-pot.sh`
- [ ] 6.3 Accessible labels on all interactive widgets
```

---

## Bugfix Spec

Use for critical bugs where traceability is needed.

> **Tip:** for a typical bugfix workflow see `bugfix-workflow.md` (manual inclusion: `#bugfix-workflow`).

### .config.kiro

```json
{"workflowType": "requirements-first", "specType": "bugfix"}
```

### requirements.md template

```markdown
# Requirements: Fix {Bug Title}

## Problem Statement
{Bug description, reproduction steps}

## Expected Behavior
{What should happen}

## Actual Behavior
{What actually happens}

## Constraints
- MUST NOT break: {list of things that must not break}
- MUST preserve: {API compatibility, data format, etc.}

## Acceptance Criteria
- [ ] Bug no longer reproduces
- [ ] Existing tests pass
- [ ] New regression test added
- [ ] No clippy warnings
```

### tasks.md template

```markdown
# Tasks: Fix {Bug Title}

## Task 1: Reproduce
- [ ] 1.1 Write failing test that demonstrates the bug

## Task 2: Fix
- [ ] 2.1 Identify root cause
- [ ] 2.2 Implement minimal fix
- [ ] 2.3 Verify test passes

## Task 3: Verify (optional)
- [ ] 3.1 Run full test suite
- [ ] 3.2 Check related functionality not broken
- [ ] 3.3 Update CHANGELOG.md
```

---

## Refactoring Spec (Design-First)

For refactoring where you know what you want to change.

### .config.kiro

```json
{"workflowType": "design-first", "specType": "feature"}
```

Skips requirements, starts with design describing current state → desired state → migration plan.
