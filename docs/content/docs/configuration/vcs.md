---
title: Commits and tags
weight: 4
---

# Commits and tags

`bumpversion` can finish a bump by committing the rewritten files and tagging the commit. Both are **off by default** — a bare `bumpversion patch` only edits files.

```toml
[tool.bumpversion]
commit = true
tag = true
```

On the command line, `--commit` / `--no-commit` and `--tag` / `--no-tag` override the config for one run.

## The dirty-tree check

Before doing anything, `bumpversion` refuses to run if the working tree has uncommitted changes:

```text
Error:
   0: Working directory is not clean:
```

This is deliberate: a release commit should contain the version bump and nothing else. Set `allow_dirty = true`, or pass `--allow-dirty`, when you genuinely want to bundle other staged work into the release commit.

The check is skipped for the read-only commands, [`show` and `show-bump`]({{< relref "../commands/show.md" >}}).

## What gets committed

Everything the run rewrote, plus the config file, plus anything in [`additional_files`]({{< relref "files.md" >}}#extra-files-in-the-commit). The verbose report lists it explicitly before the commit message:

{{< terminal name="hooks" >}}

`CHANGELOG.md` appears in that list only because the example puts it in `additional_files` — a pre-commit hook rewrites it, but no `[[files]]` entry produces it.

## Messages and tag names

Three templates control the VCS output:

| Key | Default |
| --- | --- |
| `message` (alias `commit_message`) | `Bump version: {current_version} → {new_version}` |
| `tag_name` | `v{new_version}` |
| `tag_message` | `Bump version: {current_version} → {new_version}` |

They accept the full [placeholder set]({{< relref "reference.md" >}}#placeholders), so a Conventional Commits subject or a date-stamped tag is a one-liner:

```toml
message  = "chore(release): {current_version} → {new_version}"
tag_name = "release/{new_version}"
```

A project that tags without the `v` prefix sets `tag_name = "{new_version}"`.

If the tag already exists, the report says so and the tag is not recreated; the commit still happens.

## Signing

`sign_tags = true` (the alias `sign_tag` is also accepted) creates a signed tag, using whatever signing key Git is configured to use. The verbose report's `[tag]` block shows `sign = true` or `sign = false` for every run.

## Extra commit arguments

`commit_args` is appended to the `git commit` invocation. It is split like a shell command line:

```toml
commit_args = "--no-verify"
```

That example skips your repository's `pre-commit` Git hooks for the release commit — useful when a formatter hook would otherwise fight the bump.

## Environment passed to the commit

The `git commit` subprocess receives `BUMPVERSION_CURRENT_VERSION` and `BUMPVERSION_NEW_VERSION`, so a `commit-msg` or `prepare-commit-msg` Git hook can tell a release commit from an ordinary one. These are separate from the `BVHOOK_*` variables given to [`bumpversion`'s own hooks]({{< relref "hooks.md" >}}).
