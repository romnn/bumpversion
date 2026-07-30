---
title: Key reference
weight: 6
---

# Key reference

Every configuration key, with its type and default. Key names are written as they appear in a TOML file; the INI spelling is identical, only the section layout differs — see [Config file formats]({{< relref "formats.md" >}}#ini).

## Global keys

These live in `[tool.bumpversion]` (TOML) or `[bumpversion]` (INI).

### Version

| Key | Type | Default |
| --- | --- | --- |
| `current_version` | string | — (required in practice) |
| `parse` | regex | `(?P<major>\d+)\.(?P<minor>\d+)\.(?P<patch>\d+)` |
| `serialize` | string or list | `["{major}.{minor}.{patch}"]` |

### Search and replace

| Key | Type | Default |
| --- | --- | --- |
| `search` | template | `{current_version}` |
| `replace` | template | `{new_version}` |
| `regex` | bool | `false` |
| `ignore_missing_files` | bool | `false` |
| `ignore_missing_version` | bool | `false` |
| `no_configured_files` | bool | `false` |

### Paths

| Key | Type | Default |
| --- | --- | --- |
| `included_paths` | list of paths | — |
| `excluded_paths` | list of paths | — |
| `additional_files` | list of paths | — |

`included_paths` is a union, not a filter: it can add a path that no `[[files]]` entry produced.

### Version control

| Key | Type | Default |
| --- | --- | --- |
| `commit` | bool | `false` |
| `tag` | bool | `false` |
| `sign_tags` (alias `sign_tag`) | bool | `false` |
| `allow_dirty` | bool | `false` |
| `dry_run` | bool | `false` |
| `message` (alias `commit_message`) | template | `Bump version: {current_version} → {new_version}` |
| `tag_name` | template | `v{new_version}` |
| `tag_message` | template | `Bump version: {current_version} → {new_version}` |
| `commit_args` | string | — |

### Hooks

| Key | Type | Default |
| --- | --- | --- |
| `setup_hooks` | list of strings | `[]` |
| `pre_commit_hooks` | list of strings | `[]` |
| `post_commit_hooks` | list of strings | `[]` |

## Per-file keys

In a `[[tool.bumpversion.files]]` entry. Exactly one of `filename` and `glob` is required.

| Key | Type | Notes |
| --- | --- | --- |
| `filename` | string | A concrete path |
| `glob` | string | A glob pattern; case-insensitive |
| `glob_exclude` | string or list | Only meaningful with `glob`; TOML only |
| `parse` | regex | Overrides the global value |
| `serialize` | string or list | Overrides the global value |
| `search` | template | Overrides the global value |
| `replace` | template | Overrides the global value |
| `regex` | bool | Overrides the global value |
| `ignore_missing_files` (alias `ignore_missing_file`) | bool | Overrides the global value |
| `ignore_missing_version` | bool | Overrides the global value |

Only these keys are per-file. `tag`, `commit`, the hooks, and the message templates are global.

## Per-component keys

In `[tool.bumpversion.parts.<name>]`, where `<name>` is a named capture group of `parse`.

| Key | Type | Notes |
| --- | --- | --- |
| `values` | list of strings | Allowed values, in order. Without it the component is numeric |
| `optional_value` | string | Value that may be omitted when serializing |
| `independent` | bool | Do not reset when a higher component is bumped |
| `first_value` | string | The value a reset goes to. Defaults to the first entry of `values`, or `0` |
| `always_increment` | bool | Increment the component on every bump |
| `depends_on` | string | The component this one resets with |
| `calver_format` | string | Reserved; CalVer formatting is not implemented |

## Placeholders

Available in `serialize`, `search`, `replace`, `tag_name`, `tag_message`, and `message`.

### Versions

| Placeholder | Value |
| --- | --- |
| `{current_version}` | Version before the bump |
| `{new_version}` | Version after the bump |
| `{current_<part>}` | One per component — `{current_major}`, `{current_minor}`, … |
| `{new_<part>}` | One per component — `{new_major}`, … |

Inside a `serialize` pattern, the bare component names are also available: `{major}`, `{minor}`, `{patch}`, and any other capture group of `parse`.

### Repository

| Placeholder | Value |
| --- | --- |
| `{tool}` | Always `git` |
| `{commit_sha}` | Current commit |
| `{distance_to_latest_tag}` | Commits since the most recent tag |
| `{current_tag}` | Most recent tag |
| `{branch_name}` | Current branch |
| `{short_branch_name}` | Branch name, shortened |
| `{repository_root}` | Absolute path to the repository root |
| `{dirty}` | `true` or `false` |

### Time and environment

| Placeholder | Value |
| --- | --- |
| `{now}` | Local time, RFC 3339 |
| `{utcnow}` | UTC, RFC 3339 |
| `{$VAR}` | Any environment variable, prefixed with `$` — for example `{$CI_PIPELINE_ID}` |

`{now}` and `{utcnow}` accept a [chrono](https://docs.rs/chrono/latest/chrono/format/strftime/index.html) format after a colon:

```toml
tag_name = "nightly-{utcnow:%Y%m%d}"
```

### Escapes

| Placeholder | Value |
| --- | --- |
| `{#}` | A literal `#` |
| `{;}` | A literal `;` |

These exist because `#` and `;` start a comment in INI files. Write `{{` and `}}` for literal braces.

An **unknown placeholder is an error**, not an empty string — a typo in a template fails the run rather than silently producing a wrong tag.

## Environment variables

Every command-line flag has a matching environment variable, named `BUMPVERSION_` plus the flag in upper snake case — `--dry-run` is `BUMPVERSION_DRY_RUN`, `--tag-name` is `BUMPVERSION_TAG_NAME`. The full list is in the [CLI reference]({{< relref "../commands/cli-reference.md" >}}).
