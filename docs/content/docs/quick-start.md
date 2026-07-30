---
title: Quick start
weight: 3
---

# Quick start

This walks through a first config file and a first bump. It assumes `bumpversion` is [installed]({{< relref "installation.md" >}}) and that you are in a Git repository.

## 1. Write a config file

Create `.bumpversion.toml` at the repository root. It needs the current version and the list of files that mention it:

{{< example path="simple/.bumpversion.toml" >}}

That is the whole configuration for a project whose version appears in `Cargo.toml` and in the `README`:

{{< example path="simple/Cargo.toml" >}}
{{< example path="simple/README.md" >}}

You do not list `.bumpversion.toml` itself — its `current_version` is always updated as part of the bump.

## 2. See what the next version would be

```bash
bumpversion show-bump patch
```

`show-bump` computes the next version and prints it. It writes nothing and touches no files:

{{< terminal name="show-bump" >}}

Note what `major` does: `minor` and `patch` both reset to zero. Every component below the one you bump resets, which is what makes `1.4.2` become `2.0.0` rather than `2.4.2`.

## 3. Preview the whole bump

```bash
bumpversion --dry-run --verbose patch
```

This is the command worth building a habit around. `--dry-run` gates every write, and `--verbose` prints the report — so you see the exact diff of every file, the commit that would be made, and the tag that would be created, before anything happens:

{{< terminal name="bump" >}}

Read it top to bottom:

- **`[current version]`** and **`[new version]`** — the version before and after. The second line under each (shown here because the run used `-vv`) breaks the version into its components.
- **One block per file** — the resolved `search` and `replace` templates, then a unified diff of the change. Note that `.bumpversion.toml` appears even though it is not in `[[files]]`.
- **`[commit]`** — every file that would be staged, and the commit message.
- **`[tag]`** — the tag name, its message, and whether it would be signed.
- **`[setup]`**, **`[pre-commit]`**, **`[post-commit]`** — the [hooks]({{< relref "configuration/hooks.md" >}}), or a note that none are configured.

The ` [DRY-RUN] ` prefix on every line is exactly that: it appears only because this was a dry run.

> [!WARNING]
> Without `--verbose`, a successful bump prints **nothing at all**. That is the default, and it surprises most people the first time. See [Verbosity]({{< relref "commands/cli-reference.md" >}}#verbosity).

## 4. Do it

```bash
bumpversion patch
```

The files are rewritten. Because the example config sets `commit = true` and `tag = true`, the changes are also committed and tagged as `v1.4.3`. Both default to `false` — see [Commits and tags]({{< relref "configuration/vcs.md" >}}).

`bumpversion` refuses to run on a dirty working tree, so the release commit contains only the version bump. Pass `--allow-dirty` when you mean to include other staged work.

## 5. Read values back

`show` prints resolved configuration and repository state — useful in scripts and for working out why a template rendered the way it did:

{{< terminal name="show" >}}

One variable prints a bare value; two or more print `name=value` lines, so the output is easy to consume from a shell.

## Where to go next

- The version scheme is `1.4.2` only because that is the default. A pre-release ladder or a date-stamped scheme is a `parse` regex and a couple of `serialize` patterns — see [Version scheme]({{< relref "configuration/versioning.md" >}}).
- Rewriting the same string across many packages is one `glob` entry — see [Files to rewrite]({{< relref "configuration/files.md" >}}).
- If your project already has a `pyproject.toml` or a `setup.cfg` with a `bumpversion` section, you do not need `.bumpversion.toml` at all — see [Config file formats]({{< relref "configuration/formats.md" >}}).
