# rustconn-cli

Headless management of RustConn data over `rustconn-core`. Root `AGENTS.md` still
applies, with one inversion you need to know before you "fix" anything here.

## `println!` is the interface, not a leftover

The root rules say logging goes through `tracing` and that `println!`/`eprintln!`
are debug leftovers to be removed. In this crate they are the product. `main.rs`
carries three crate-level allows, each with a reason:

- `clippy::print_stdout` — data output
- `clippy::print_stderr` — warnings and errors
- `unreachable_pub` — `pub` is inter-module visibility in a binary crate

So `println!` in `commands/` is correct and must not be converted to `tracing`.
What still applies: `tracing::error!` for the diagnostic record, `eprintln!` only
for what the user must see, and both in `main` gated on `--quiet`.

## The rest

- **Only `rustconn-core`.** No `gtk4`, no `adw`, no `vte4` — a pre-write hook
  rejects the edit. Any other workspace crate as a dependency needs a reason in
  the commit message.
- Default features minimal. Anything pulling a runtime integration goes behind a
  feature flag, as in core.
- Errors are `thiserror` (`src/error.rs`), not `anyhow`, even though M-APP-ERROR
  would permit `anyhow` in a binary. The reason is `exit_code()`: the process exit
  status is derived from the variant, so a new failure mode means a new variant
  with a deliberate code, not a stringly-typed context chain.
- A command that lists or shows data takes `--format` with `table` (default),
  `json` and `csv`, and implements all three. `commands/cluster.rs` is the
  reference shape; copy it rather than inventing a fourth output style. Dispatch
  on `format.effective()`, not on the raw value: `table` becomes `json` when
  stdout is not a terminal, so a piped or redirected command emits structured
  output (clig.dev). Matching the raw value drops that silently — the flag still
  says `table` and nothing fails.
- **This crate is English-only, and that is a gap rather than a decision.** The
  line that used to be here said user-facing text goes through `i18n()` /
  `i18n_f()` and that "a CLI is not exempt from the locales". Check it:
  `grep -rn i18n rustconn-cli/src` returns nothing, and never has. Every
  `println!` in `commands/` is a bare English literal, and none of the crate's
  files are in `po/POTFILES.in`.

  So do not "follow the rule" by wrapping the one string you happen to be
  touching. A single `i18n()` call among several hundred bare literals gives a
  translator one string out of a screenful and makes the output a mix of two
  languages — worse than consistent English. Translating this crate is a task of
  its own: every literal at once, the files added to `POTFILES.in`, and a decision
  about machine-readable output, which must stay stable regardless of locale
  (`--format json` and `--format csv` are parsed by scripts). Until someone does
  that, new strings here are English.

Surface reference: `docs/CLI_REFERENCE.md`.
