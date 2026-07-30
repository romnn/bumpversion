# Agent guidelines

Project-specific instructions for coding agents working on `bumpversion`.

## Layout

- `crates/bumpversion` — the library. Config parsing (`config/`), version parsing
  and bumping (`version.rs`), file search and replace (`files.rs`), VCS backends
  (`vcs/`), hooks, and diagnostics.
- `crates/bumpversion-cli` — the `bumpversion` and `cargo-bumpversion` binaries.
  Argument parsing lives in `options.rs`, shared setup in `common.rs`.

## Rust error handling in tests

Do not scatter imports from `color_eyre::eyre::*` throughout test code. Import
the module once at the top of each test module:

```rust
use color_eyre::eyre;
```

When extension traits or other items are also needed, group them into the same
import:

```rust
use color_eyre::eyre::{self, OptionExt as _, ...};
```

Then qualify macros and other APIs through the module, such as `eyre::eyre!`,
`eyre::bail!`, and `eyre::Result`.

Library code must use typed errors derived with `thiserror`; library crates may
depend on `color-eyre` only as a dev-dependency. Binary crates may use
`color-eyre` as a regular dependency, but production references to it belong
only in the binary's `main` entrypoint. Binary helper modules must also return
typed `thiserror` errors. Test modules are the explicit exception and may use
the dev-only `color_eyre::eyre` re-export described above.

## Multiline strings

Use [`indoc!`](https://docs.rs/indoc) for multiline string literals and
`formatdoc!` when the literal interpolates values. Both strip the leading
indentation, which keeps the literal aligned with the surrounding code instead
of forcing it back to column zero:

```rust
let config = indoc! {r#"
    [tool.bumpversion]
    current_version = "1.2.3"
"#};

let message = formatdoc! {"
    bumped {current} to {new}
"};
```

This matters most in the config tests, where the fixtures are TOML and INI
documents whose own indentation is significant. Prefer these macros over
`concat!`, manual `\n` joins, or a literal that escapes the block's indentation.

This applies to ordinary and raw string literals. A string split across source
lines with Rust's trailing-backslash continuation does not contain a line break
and does not need either macro.

Keep a direct literal where Rust syntax or an outer macro requires a literal
token, such as an attribute, a pattern, or a macro argument matched as
`$literal`. `indoc!` and `formatdoc!` expand as expressions and cannot replace
those forms.

## Commands

Run these through the taskfile rather than invoking cargo directly — the tasks
carry the flags CI uses.

- `task test` — run the workspace test suite (nextest).
- `task test:fc` — run it across the feature-combination matrix, as CI does.
- `task lint` — clippy over all targets. The workspace denies `clippy::pedantic`,
  `unwrap_used`, `expect_used`, `panic`, and `indexing_slicing`.
- `task format` — rustfmt.
- `task run -- <args>` — run the CLI.

## Portability

CI tests on Linux, macOS, and Windows. Assertions on CLI output must not hard-code
the binary name — clap renders it from `argv[0]`, which is `bumpversion.exe` on
Windows. See the `usage` helper in `crates/bumpversion-cli/tests/test_cli.rs`.
