---
title: bumpversion
type: docs
bookToc: false
---

<div class="bv-hero">
  <div class="bv-hero__text">
    <h1>bumpversion</h1>
    <p class="bv-hero__lead">Update <strong>every version string</strong> in your project, then commit and tag the change — one command, one config file. A fast, dependency-free Rust rewrite of <code>bump-my-version</code> that reads the config you already have.</p>
    <div class="bv-hero__cmd">bumpversion patch</div>
    <div class="bv-hero__actions">
      <a class="bv-btn bv-btn--primary" href="{{< relref "/docs/introduction.md" >}}">Read the docs</a>
      <a class="bv-btn" href="https://github.com/romnn/bumpversion">Source on GitHub</a>
    </div>
  </div>
</div>

<div class="bv-badges">

[![build status](https://img.shields.io/github/actions/workflow/status/romnn/bumpversion/build.yaml?branch=main&label=build)](https://github.com/romnn/bumpversion/actions/workflows/build.yaml)
[![test status](https://img.shields.io/github/actions/workflow/status/romnn/bumpversion/test.yaml?branch=main&label=test)](https://github.com/romnn/bumpversion/actions/workflows/test.yaml)
[![crates.io](https://img.shields.io/crates/v/bumpversion)](https://crates.io/crates/bumpversion)
[![docs.rs](https://img.shields.io/docsrs/bumpversion/latest?label=docs.rs)](https://docs.rs/bumpversion)

</div>

## Why

A release touches the same number in a dozen places: the package manifest, a `README` install line, a constant in the source, a docs heading. Doing that by hand is how a release ships with one of them stale.

`bumpversion` reads the current version from one config file, works out the next one, rewrites every place you listed, and commits and tags the result — atomically, and with a `--dry-run` that shows you the exact diff first.

It is a drop-in rewrite of [`bump-my-version`](https://github.com/callowayproject/bump-my-version) (formerly `bumpversion` / `bump2version`): the same `.bumpversion.toml`, `pyproject.toml`, and `setup.cfg` layouts, without a Python install in the loop.

<div class="bv-cards">
  <div class="bv-card">
    <h3>No Python required</h3>
    <p>A single static binary from <code>brew</code>, <code>cargo</code>, or a release download — no global <code>pip install</code>, no virtualenv to activate before you can cut a release.</p>
  </div>
  <div class="bv-card">
    <h3>Your existing config</h3>
    <p>Reads <code>.bumpversion.toml</code>, <code>pyproject.toml</code>, <code>.bumpversion.cfg</code>, and <code>setup.cfg</code> — point it at a project you already have and it works.</p>
  </div>
  <div class="bv-card">
    <h3>Any version scheme</h3>
    <p>A <code>parse</code> regex and <code>serialize</code> patterns describe the scheme, so SemVer, pre-release ladders, and date-stamped builds are all just configuration.</p>
  </div>
  <div class="bv-card">
    <h3>Show before you commit</h3>
    <p><code>--dry-run --verbose</code> prints the full diff of every file, the commit it would make, and the tag it would create — before anything is written.</p>
  </div>
</div>

## Example

```bash
# Install a prebuilt binary
brew install --cask romnn/tap/bumpversion

# What would the next version be?
bumpversion show-bump patch

# Show every change a patch release would make, without making it
bumpversion --dry-run --verbose patch

# Do it: rewrite the files, commit, tag
bumpversion patch
```

The whole configuration for a project whose version lives in `Cargo.toml` and the `README`:

{{< example path="simple/.bumpversion.toml" >}}

## See it before you run it

`--dry-run --verbose` prints the entire result of a bump — the diff of every file, the commit it would make, the tag it would create — without writing anything:

<div class="bv-showcase">
{{< terminal name="bump" >}}
</div>

## Documentation

- [Introduction]({{< relref "/docs/introduction.md" >}}) and [Installation]({{< relref "/docs/installation.md" >}}).
- [Quick start]({{< relref "/docs/quick-start.md" >}}) — a first bump and how to read the report.
- [Configuration]({{< relref "/docs/configuration/_index.md" >}}) — the files to rewrite, the version scheme, commits, tags, and hooks.
- [Commands]({{< relref "/docs/commands/_index.md" >}}) — `bump`, `show`, `show-bump`, and the full flag reference.
- [Continuous integration]({{< relref "/docs/ci.md" >}}) — cutting releases from GitHub Actions.
