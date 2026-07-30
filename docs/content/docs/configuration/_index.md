---
title: Configuration
weight: 4
bookCollapseSection: true
---

# Configuration

Everything `bumpversion` does comes from one config file at the root of your repository. The minimum is a current version and the files that mention it:

```toml
[tool.bumpversion]
current_version = "1.4.2"

[[tool.bumpversion.files]]
filename = "Cargo.toml"
```

From there, configuration falls into five areas:

- **[Config file formats]({{< relref "formats.md" >}})** — which files are searched, in what order, and how the TOML and legacy INI layouts differ.
- **[Files to rewrite]({{< relref "files.md" >}})** — `[[files]]` entries, `filename` against `glob`, and narrowing a match with `search` and `replace`.
- **[Version scheme]({{< relref "versioning.md" >}})** — the `parse` regex and `serialize` patterns that define what a version *is*, plus optional components for pre-release ladders.
- **[Commits and tags]({{< relref "vcs.md" >}})** — what gets committed, the message and tag templates, and the dirty-tree check.
- **[Hooks]({{< relref "hooks.md" >}})** — running commands before and after the bump, and the `BVHOOK_*` variables they receive.

The **[key reference]({{< relref "reference.md" >}})** lists every key with its type and default, and every placeholder available to a template.

> [!NOTE]
> Command-line flags override config file values for that run. Where a key is a boolean, the flag pair is `--x` / `--no-x` — for example `--tag` and `--no-tag` override `tag`.
