---
title: Introduction
weight: 1
---

# Introduction

`bumpversion` answers one question: *this project is at version X — make it version Y, everywhere.*

"Everywhere" is the hard part. A version number is duplicated across a package manifest, an install line in the `README`, a constant in the source, a heading in the docs, and the config file that records the current version. Bumping by hand means finding all of them, and a release where one was missed is a release with a wrong number in it.

## What a bump does

Every run follows the same sequence. The [verbose report]({{< relref "commands/bump.md" >}}#the-verbose-report) is structured to mirror it, so the output reads as a trace of these steps:

1. **Read the config.** The first recognized [config file]({{< relref "configuration/formats.md" >}}) in the working directory wins, and command-line flags override its values.
2. **Parse the current version.** The `parse` regex splits `current_version` into named components — by default `major`, `minor`, and `patch`.
3. **Run the setup hooks.** If any [`setup_hooks`]({{< relref "configuration/hooks.md" >}}) are configured, they run before anything is touched, and a non-zero exit aborts the bump.
4. **Compute the new version.** The requested component is incremented and every component below it resets. The result is turned back into a string by the first `serialize` pattern that fits.
5. **Rewrite the files.** For each configured file, the rendered `search` template is located and replaced with the rendered `replace` template. The config file's own `current_version` is updated too.
6. **Run the pre-commit hooks**, then **commit**, then **tag**, then run the **post-commit hooks** — each step only if enabled.

Nothing is written until step 5, which is why `--dry-run` can show you the complete result — including the diff of every file, the commit message, and the tag — without side effects.

## Components and resets

The component names come from the named capture groups in `parse`. The default pattern

```text
(?P<major>\d+)\.(?P<minor>\d+)\.(?P<patch>\d+)
```

produces `major`, `minor`, and `patch`, which is why `bumpversion minor` on `1.4.2` gives `1.5.0` — `minor` increments, `patch` resets to zero.

Change `parse` and you change the scheme. Adding an optional pre-release group gives you a component ladder to walk before a release; see [Version scheme]({{< relref "configuration/versioning.md" >}}).

## Relationship to bump-my-version

This is a rewrite of [`callowayproject/bump-my-version`](https://github.com/callowayproject/bump-my-version), itself the successor to `peritus/bumpversion` and `c4urself/bump2version`. It reads the same configuration files and uses the same key names, template placeholders, and version-scheme model, so pointing it at an existing project generally works with no edits.

Two things differ, and both are the point of the rewrite:

- **Distribution.** A single static binary, installable with `brew` or `cargo` or downloaded from a release. Release tooling that needs no Python on the machine is much easier to put in a container or a CI job.
- **Speed.** No interpreter start-up on a command that runs at every release.

A few `bump-my-version` features are still missing — configuration in `Cargo.toml`, floating major/minor tags, and CalVer formatting — but the configuration surface documented here is implemented.

## Where to go next

[Install it]({{< relref "installation.md" >}}), then follow the [quick start]({{< relref "quick-start.md" >}}) to write a first config and run a first bump.
