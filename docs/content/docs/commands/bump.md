---
title: Bumping
weight: 1
---

# Bumping

## Selecting a component

There are three equivalent ways to bump, and one way to bypass component selection entirely:

```bash
bumpversion patch                       # shorthand for the standard components
bumpversion bump patch                  # explicit; works for any component
bumpversion bump pre_label              # a component you defined in `parse`
bumpversion --new-version 2.0.0-rc.1    # set the version directly
```

`major`, `minor`, and `patch` exist as their own commands because they are what most projects use. Any other component — anything with a named capture group in your [`parse` pattern]({{< relref "../configuration/versioning.md" >}}) — goes through `bump <component>`.

`--new-version` skips the version arithmetic and uses the string you give it, for a release that does not follow from incrementing anything.

Trailing arguments after the component are treated as file paths to restrict the run to:

```bash
bumpversion bump patch docs/index.md
```

## Preview first

`--dry-run` (or `-n`) gates every write — no file is touched, no commit is made, no hook runs. Combined with `--verbose` it prints the complete result of the bump that would have happened:

```bash
bumpversion --dry-run --verbose patch
```

{{< terminal name="bump" >}}

## The verbose report

The report follows the [order of a bump]({{< relref "../introduction.md" >}}#what-a-bump-does), so it reads as a trace:

**`[current version]`** and **`[new version]`** — the version before and after. With `-vv`, a second line breaks each into components (`major=1  minor=4  patch=2`), which is the fastest way to see why a bump produced the number it did.

**One block per file** — the file's absolute path, then the resolved templates:

```text
replacing `{current_version}` (1.4.2) with `{new_version}` (1.4.3)
```

The template is shown alongside the concrete string it rendered to, so a `search` that matched nothing is obvious. Below that is a unified diff of the change, or `no changes` if the file already matched.

The config file appears in this list even though it is not in `[[files]]` — its `current_version` is always updated.

**`[setup]`**, **`[pre-commit]`**, **`[post-commit]`** — the [hooks]({{< relref "../configuration/hooks.md" >}}) that ran, or `no ... hooks defined`.

**`[commit]`** — every file staged, then the rendered commit message.

**`[tag]`** — the tag name, its message, and whether it is signed. If the tag already exists, a note that it will not be created.

Under `--dry-run` every line carries a ` [DRY-RUN] ` prefix, so a preview can never be mistaken for a record of something that happened.

> [!WARNING]
> Without `-v`, a bump prints **nothing**, successful or not. See [Verbosity]({{< relref "cli-reference.md" >}}#verbosity).

## Overriding config for one run

Any config key with a command-line equivalent can be overridden per run. The boolean pairs are the useful ones:

```bash
# Rewrite the files but do not commit or tag
bumpversion patch --no-commit --no-tag

# Commit and tag even though the config does not
bumpversion patch --commit --tag

# Proceed despite an unclean working tree
bumpversion patch --allow-dirty
```

`--current-version` and `--new-version` override the versions; `--parse` and `--serialize` override the scheme; `--search` and `--replace` override the templates; `--tag-name` and `--message` override the VCS templates.

## Safety

Two behaviors are worth relying on:

- **The tree must be clean.** A bump aborts if there are uncommitted changes, so the release commit contains only the version change. `--allow-dirty` opts out.
- **A missing match is an error.** A configured file that does not exist, or that does not contain the current version, fails the run instead of silently producing a partial bump. [`ignore_missing_files` and `ignore_missing_version`]({{< relref "../configuration/files.md" >}}#missing-files-and-missing-versions) opt out per file.

A failed hook aborts the bump. Because `post_commit_hooks` run after the commit and tag exist, a failure there cannot roll them back.
