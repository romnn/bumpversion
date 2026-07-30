---
title: CLI reference
weight: 3
---

# CLI reference

{{< terminal name="help" >}}

Every flag below is **global**: accepted at the top level and on every subcommand alike. Each also has an environment variable, named `BUMPVERSION_` plus the flag in upper snake case.

## General

| Flag | Value | Description |
| --- | --- | --- |
| `--dir` | path | Repository directory to run in |
| `--color` | `auto`, `always`, `always-ansi`, `never` | Enable or disable color. Defaults to `auto`, which is on only when stdout is a terminal |
| `-v`, `--verbose` | repeatable | Increase verbosity |
| `-q`, `--quiet` | repeatable | Decrease verbosity |
| `--log` | level | Tracing log level. Also accepted as `--log-level` |
| `-h`, `--help` | | Print help |
| `-V`, `--version` | | Print version |

## Version

| Flag | Value | Description |
| --- | --- | --- |
| `--current-version` | string | Version to bump from, overriding the config |
| `--new-version` | string | Version to write, skipping component selection |
| `--parse` | regex | Regex parsing the version string |
| `--serialize` | string | How to format components back into a version. Repeatable |

## Files

| Flag | Value | Description |
| --- | --- | --- |
| `--search` | template | String to search for |
| `--replace` | template | String to replace it with |
| `--no-configured-files` | flag | Only rewrite files named on the command line |
| `--ignore-missing-files` / `--no-ignore-missing-files` | flag | Whether a missing file is an error |
| `--ignore-missing-version` / `--no-ignore-missing-version` | flag | Whether a missing version in a file is an error |

## Version control

| Flag | Value | Description |
| --- | --- | --- |
| `-n`, `--dry-run` | flag | Write nothing; just report |
| `--allow-dirty` / `--no-allow-dirty` | flag | Whether to proceed on an unclean working tree |
| `--commit` / `--no-commit` | flag | Whether to commit |
| `--tag` / `--no-tag` | flag | Whether to tag |
| `--sign-tags` / `--no-sign-tags` | flag | Whether to sign the tag |
| `--tag-name` | template | Tag name |
| `-m`, `--message` | template | Commit message |
| `--commit-args` | string | Extra arguments for `git commit` |

Each `--x` / `--no-x` pair overrides the corresponding config key for one run; without either, the config value stands.

## Verbosity

The report is off by default. This is the single most surprising thing about the tool:

| Flags | Output |
| --- | --- |
| *(none)* | **Nothing.** A successful bump is completely silent |
| `-v` | The full report: versions, per-file diffs, commit, tag, hooks |
| `-vv` | Adds the per-component breakdown under each version |
| `-vvv` and above | Accepted, identical to `-vv` |
| `-q` | Same as the default |

`-q` and `-v` cannot be combined; doing so is a usage error.

`--log` is separate: it controls tracing diagnostics from the library, not the report. It defaults to `WARN`, which is why a mismatch between the configured version and the last tag shows up even in an otherwise silent run:

```text
WARN bumpversion::common: version 1.4.2 from config does not match last tagged version (1.4.3)
```

`RUST_LOG` overrides `--log` when set, taking the same directive syntax as any `tracing` filter — `RUST_LOG=bumpversion=debug`.

## Exit codes

| Code | Meaning |
| --- | --- |
| `0` | Success. Also `--help`, `--version`, and a `show` with an unknown variable |
| `1` | The run failed — no config file, unclean tree, unknown component, a hook that failed |
| `2` | The command line could not be parsed — unknown flag, bad `--color` value, missing component |

Nothing finer distinguishes the failure modes, so a script that needs to tell "dirty tree" from "missing config" has to match on the message.

Common exit-1 messages:

| Message | Cause |
| --- | --- |
| `missing config file` | No recognized config file in the directory |
| `Working directory is not clean:` | Uncommitted changes; pass `--allow-dirty` |
| `missing version component to bump` | No component given and no `--new-version` |
| `missing current version` | No `current_version` in the config or on the command line |
| `failed to parse current version` | `current_version` does not match the `parse` pattern |
| `the component has already the maximum value ...` | A `values` component is already at its last entry |

## Requirements

`bumpversion` opens the Git repository before doing anything, so **every** command — including `show` — must run inside one.
