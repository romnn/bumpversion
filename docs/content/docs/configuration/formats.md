---
title: Config file formats
weight: 1
---

# Config file formats

`bumpversion` looks for its configuration in the directory it runs in — the current directory, or the one given by `--dir`.

## Discovery order

Four filenames are tried, in this order:

| # | File | Section |
| --- | --- | --- |
| 1 | `.bumpversion.toml` | `[tool.bumpversion]` |
| 2 | `.bumpversion.cfg` | `[bumpversion]` |
| 3 | `pyproject.toml` | `[tool.bumpversion]` |
| 4 | `setup.cfg` | `[bumpversion]` |

The **first file that contains a usable section wins**, and only that file is used — configuration is never merged across files. A `pyproject.toml` with no `[tool.bumpversion]` table (or with an empty one) is skipped as though it were not there, so the search continues to `setup.cfg`.

If no file yields a configuration, the run fails with `missing config file`.

> [!NOTE]
> `Cargo.toml` is checked last but reading configuration from it is **not implemented** — a `[package.metadata.bumpversion]` table has no effect today.

## TOML

The native format. Everything lives under `[tool.bumpversion]`, per-file entries are an array of tables, and per-component settings are nested tables:

{{< example path="pyproject/pyproject.toml" >}}

Running against it produces:

{{< terminal name="pyproject" >}}

Two details worth noting in that output:

- `[project].version` and `[tool.bumpversion].current_version` are both rewritten, because `pyproject.toml` is listed as a file *and* is the config file.
- The tag name and commit message come from the `tag_name` and `message` templates in the config.

### Regexes in TOML

The `parse` key is a regex, and regexes are full of backslashes. Use a TOML **literal string** (single quotes) so they are taken as written:

```toml
parse = '(?P<major>\d+)\.(?P<minor>\d+)\.(?P<patch>\d+)'
```

In a basic string (double quotes) every backslash has to be doubled, which is easy to get wrong:

```toml
parse = "(?P<major>\\d+)\\.(?P<minor>\\d+)\\.(?P<patch>\\d+)"
```

## INI

`.bumpversion.cfg` and `setup.cfg` use the legacy INI layout from `bump2version`, supported so an existing project works unchanged:

{{< example path="setup-cfg/setup.cfg" >}}

The differences from TOML:

- **Global keys** go in `[bumpversion]`.
- **Per-file entries** are their own sections, named `[bumpversion:file:<path>]`. A disambiguating suffix is allowed, so two entries can target the same file: `[bumpversion:file(version heading):CHANGELOG.md]`.
- **Glob entries** are `[bumpversion:glob:<pattern>]`. `glob_exclude` is not supported in INI.
- **Component entries** are `[bumpversion:part:<name>]`.
- **Values are unquoted**, and booleans accept `True` / `False` as well as `true` / `false`.
- **Lists** are written one item per line (indented), or comma-separated on one line:

  ```ini
  serialize =
      {major}.{minor}.{patch}-{release}
      {major}.{minor}.{patch}
  ```

- The literal value `None` means "unset" for most keys.
- In `[bumpversion:part:<name>]`, `values` must be multi-line or comma-separated — a bare single value is not accepted.

In `setup.cfg`, sections unrelated to `bumpversion` are ignored silently. In `.bumpversion.cfg` — a file that exists only for this tool — an unrecognized section is reported as a diagnostic.

> [!NOTE]
> The INI parser currently prints a few `=> section: …` debug lines to stdout while reading the file. They come from the underlying [`serde-ini-spanned`](https://github.com/romnn/serde-ini-spanned) parser and are harmless, but they do clutter the output — TOML is the quieter choice for a new project.

## Choosing a file

- **New project** — `.bumpversion.toml`. It is found first and keeps release configuration out of your package manifest.
- **Python project** — `[tool.bumpversion]` in `pyproject.toml`, next to the rest of your tooling.
- **Existing `bump2version` project** — leave `setup.cfg` or `.bumpversion.cfg` where it is; it is read as-is.
