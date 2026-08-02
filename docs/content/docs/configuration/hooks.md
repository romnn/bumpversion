---
title: Hooks
weight: 5
---

# Hooks

Hooks run shell commands at three points in a bump. Each is a list of command strings:

| Key | Runs |
| --- | --- |
| `setup_hooks` | Before anything is read or written |
| `pre_commit_hooks` | After the files are rewritten, before `git add` and the commit |
| `post_commit_hooks` | After the commit and the tag |

Every hook runs through `sh -c` from the repository root, and **a non-zero exit aborts the bump**. Hooks do not run under `--dry-run` — the report shows what would have run:

{{< example path="hooks/.bumpversion.toml" >}}

{{< terminal name="hooks" >}}

## What each is for

**`setup_hooks`** are preconditions. Because they run before any file is touched, a failure costs nothing. The example above asserts the tree is clean; a release guard that refuses to ship from the wrong branch is the same shape:

```toml
setup_hooks = ['test "$BVHOOK_BRANCH_NAME" = main']
```

**`pre_commit_hooks`** are for files that must be regenerated *from* the new version and land in the same commit — a lockfile whose package version just changed, or a changelog heading:

```toml
pre_commit_hooks = ["cargo metadata --offline --format-version 1 >/dev/null"]
additional_files = ["Cargo.lock"]
```

Anything a pre-commit hook writes that no `[[files]]` entry produced must also be listed in [`additional_files`]({{< relref "files.md" >}}#extra-files-in-the-commit), or it will not be staged. That lockfile case is worked through in full [below](#rust-keeping-cargolock-in-the-release-commit).

**`post_commit_hooks`** run once the release exists — publishing, notifying, or kicking off a build. A failure here aborts the run but cannot undo the commit and tag that already happened.

## Environment

Hooks inherit the full environment, plus these `BVHOOK_*` variables:

| Variable | Value |
| --- | --- |
| `BVHOOK_NOW` | Local time, RFC 3339 |
| `BVHOOK_UTCNOW` | UTC, RFC 3339 |
| `BVHOOK_COMMIT_SHA` | Current commit |
| `BVHOOK_DISTANCE_TO_LATEST_TAG` | Commits since the most recent tag |
| `BVHOOK_IS_DIRTY` | `true` or `false` |
| `BVHOOK_CURRENT_VERSION` | Version before the bump |
| `BVHOOK_CURRENT_TAG` | Most recent tag |
| `BVHOOK_BRANCH_NAME` | Current branch |
| `BVHOOK_SHORT_BRANCH_NAME` | Branch name, shortened |
| `BVHOOK_CURRENT_<PART>` | One per component — `BVHOOK_CURRENT_MAJOR`, `BVHOOK_CURRENT_MINOR`, … |

`pre_commit_hooks` and `post_commit_hooks` additionally get:

| Variable | Value |
| --- | --- |
| `BVHOOK_NEW_VERSION` | The new version |
| `BVHOOK_NEW_<PART>` | One per component — `BVHOOK_NEW_MAJOR`, … |
| `BVHOOK_NEW_VERSION_TAG` | The tag this bump will create, rendered from `tag_name`. Empty when `tag` is off |

Setup hooks do not get the `NEW_*` variables, because the new version has not been computed yet.

## Writing a hook

Because a hook is a single string handed to `sh -c`, anything beyond one command reads better in a script file. The example keeps the changelog rewrite in `scripts/changelog.sh` and passes the version as an argument:

{{< example path="hooks/scripts/changelog.sh" lang="bash" >}}

Keep hooks idempotent where you can. A bump that fails partway leaves the earlier hooks' effects in place, and the natural response is to fix the problem and run it again.

## Rust: keeping Cargo.lock in the release commit

Bumping the version in `Cargo.toml` leaves `Cargo.lock` stale — it still records the workspace crates at the old version — and most projects want both in one commit. Any cargo command that resolves the workspace rewrites the lockfile, so the job needs nothing more than the cheapest one:

```toml
[tool.bumpversion]
current_version = "1.4.2"
commit = true
tag = true

pre_commit_hooks = ["cargo metadata --offline --format-version 1 >/dev/null"]
additional_files = ["Cargo.lock"]

[[tool.bumpversion.files]]
filename = "Cargo.toml"
```

`cargo metadata` resolves the workspace, writes the refreshed lockfile, and prints a JSON dump the hook throws away. `--format-version 1` silences cargo's warning about the unpinned output format, and `--offline` keeps the hook off the network. Drop `--offline` if the bump may run somewhere the dependency cache is cold.

> [!WARNING]
> Do not reach for `cargo update` here. It re-resolves **every** dependency to the newest version your requirements allow, so a release bump quietly becomes a dependency bump — which defeats a pinning or cooldown policy meant to slow supply-chain attacks down.
>
> It can also fail outright: re-resolving an unpinned git dependency looks at the cached checkout's branch tip, which may no longer contain the package the lockfile pins. The bump then aborts *after* the files were rewritten, leaving the new version in the tree with no commit.
>
> `cargo update --workspace --offline` is the narrow form and does restrict itself to the workspace crates, but it is still an update command. `cargo metadata` cannot move a dependency even by accident.
