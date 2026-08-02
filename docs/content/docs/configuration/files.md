---
title: Files to rewrite
weight: 2
---

# Files to rewrite

Each `[[tool.bumpversion.files]]` entry describes one place the version appears. An entry must name exactly one target — either a `filename` or a `glob`:

```toml
[[tool.bumpversion.files]]
filename = "Cargo.toml"

[[tool.bumpversion.files]]
glob = "packages/*/package.json"
```

Naming both, or neither, is a configuration error.

The config file itself is always rewritten and always committed — you never list it.

## What gets replaced

Every entry has a `search` template and a `replace` template. They default to:

```toml
search  = "{current_version}"
replace = "{new_version}"
```

So by default, every occurrence of the current version string in the file becomes the new one. In `README.md` that rewrites both the heading and the install line, which is usually what you want:

{{< example path="simple/README.md" >}}

`search` is matched **literally** by default — the rendered template is escaped before use, so a version like `1.4.2` cannot have its dots act as regex wildcards.

## Narrowing the match

Bare `{current_version}` is too broad when the same string appears somewhere it should not change — a pinned dependency that happens to share your version number, or a changelog entry for a past release. Give the entry enough surrounding context to be unambiguous:

```toml
[[tool.bumpversion.files]]
filename = "package.json"
search   = '"version": "{current_version}"'
replace  = '"version": "{new_version}"'
```

Now only the manifest's own `version` key matches.

`search` and `replace` are per-file overrides of the global keys, so you can set a project-wide default and override it for one file.

> [!WARNING]
> The rendered `replace` string is used as a regex replacement, so a literal `$` in it is treated as a capture-group reference. Write `$$` for a literal dollar sign.

## Globs

A `glob` entry expands to every matching file, which keeps a monorepo's config from growing a stanza per package:

{{< example path="monorepo/.bumpversion.toml" >}}

{{< terminal name="monorepo" >}}

Both packages are rewritten from one entry, and adding a third package needs no config change. Matched files are processed in sorted order.

`glob_exclude` trims matches back out, as a string or a list:

```toml
[[tool.bumpversion.files]]
glob = "packages/*/package.json"
glob_exclude = ["packages/internal/package.json"]
```

The same path may be named by more than one entry — as in the example above, where a broad entry and a narrow one both match. The changes accumulate and are applied in config order.

> [!NOTE]
> Glob matching is **case-insensitive**, and `*` crosses directory separators. Use `glob_exclude` if that pulls in more than you intended.

## Missing files and missing versions

By default, a configured file that does not exist, or that exists but does not contain the search string, is an error. That is the right default: it catches a `README` that was reorganized and no longer carries the version.

Two keys relax it, globally or per file:

| Key | Effect |
| --- | --- |
| `ignore_missing_files` | A configured file that does not exist is skipped |
| `ignore_missing_version` | A file that exists but has no match is left alone |

```toml
[[tool.bumpversion.files]]
filename = "CHANGELOG.md"
ignore_missing_version = true
```

Per-file, `ignore_missing_file` (singular) is accepted as well.

## Extra files in the commit

`additional_files` lists paths that should be **staged with the release commit but not rewritten**. The usual case is a file a [hook]({{< relref "hooks.md" >}}) regenerates — a lockfile, or a changelog whose heading a script rewrites:

```toml
additional_files = ["Cargo.lock", "CHANGELOG.md"]
```

Without this, a hook's changes would be left uncommitted in the working tree after the release commit. For a Rust project, [Cargo.lock in the release commit]({{< relref "hooks.md" >}}#rust-keeping-cargolock-in-the-release-commit) shows the whole pattern — the hook that refreshes the lockfile and the entry that stages it.

## Restricting the run

`excluded_paths` drops files from a run that the entries would otherwise produce; `included_paths` adds paths back in. Passing file paths as trailing arguments on the command line is the same as setting `included_paths`:

```bash
bumpversion patch --no-configured-files docs/index.md
```

`--no-configured-files` ignores everything in the config file, so only the paths named on the command line are touched.
