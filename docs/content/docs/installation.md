---
title: Installation
weight: 2
---

# Installation

`bumpversion` is a single static binary with no runtime dependencies.

## Homebrew

```bash
brew install --cask romnn/tap/bumpversion
```

## From crates.io

Building from source installs both binaries in the package:

```bash
cargo install --locked bumpversion-cli
```

## Prebuilt binaries

Release archives for Linux, macOS, and Windows are attached to every [GitHub release](https://github.com/romnn/bumpversion/releases). Download the archive for your platform, extract it, and put `bumpversion` on your `PATH`.

## In CI

With [mise](https://mise.jdx.dev), pin it alongside the rest of your toolchain:

```toml
[tools]
"github:romnn/bumpversion" = { version = "latest", matching = "bumpversion" }
```

The `matching` filter is needed because each release ships both the `bumpversion` and `cargo-bumpversion` archives.

## Verify

```bash
bumpversion --version
```

`bumpversion` needs to run inside a Git repository — it reads tag and branch state even for read-only commands like [`show`]({{< relref "commands/show.md" >}}).

Next: the [quick start]({{< relref "quick-start.md" >}}).
