---
title: Continuous integration
weight: 6
---

# Continuous integration

Running the bump in CI makes the release commit and tag reproducible, and keeps release rights in the repository rather than on someone's laptop.

## Manual release workflow

A `workflow_dispatch` job that takes the component to bump, does the bump, and pushes the commit and tag:

```yaml
name: release
on:
  workflow_dispatch:
    inputs:
      component:
        description: "version component to bump"
        type: choice
        options: [patch, minor, major]
        default: patch

permissions:
  contents: write

jobs:
  release:
    runs-on: ubuntu-24.04
    steps:
      - uses: actions/checkout@v7
        with:
          # bumpversion reads tag state, and the release commit is pushed back.
          fetch-depth: 0
      - uses: jdx/mise-action@v4
      - name: Configure git author
        run: |
          git config user.name "github-actions[bot]"
          git config user.email "41898282+github-actions[bot]@users.noreply.github.com"
      - name: Preview
        run: bumpversion --dry-run --verbose ${{ inputs.component }}
      - name: Bump
        run: bumpversion ${{ inputs.component }}
      - name: Push
        run: git push --follow-tags
```

Two details matter:

- **`fetch-depth: 0`.** The default shallow clone has no tags, so `{current_tag}` and `{distance_to_latest_tag}` resolve to nothing and the "does not match last tagged version" warning fires on every run.
- **A Git author must be configured.** The runner has none by default, and the commit step fails without one.

The preview step is not strictly needed, but it puts the full report in the job log, so the diff of a release is visible afterwards without checking out the commit.

With mise, pin the tool alongside the rest of the toolchain:

```toml
[tools]
"github:romnn/bumpversion" = { version = "latest", matching = "bumpversion" }
```

## Publishing on the tag

Keep bumping and publishing in separate workflows. The bump workflow pushes a tag; a second workflow triggers on it:

```yaml
name: publish
on:
  push:
    tags: ["v*"]
```

That separation means a failed publish can be re-run against the same tag without producing a second version bump.

Note that a push from `GITHUB_TOKEN` does not trigger other workflows by default. Either use a PAT or a GitHub App token for the push, or dispatch the publish workflow explicitly.

## Checking the bump on pull requests

A dry run is a cheap guard that the config still matches the repository — it fails if a configured file went missing or stopped containing the version:

```yaml
- name: Verify the release config still applies
  run: bumpversion --dry-run --verbose --allow-dirty patch
```

`--allow-dirty` is there because the check should not depend on the tree being clean; `--dry-run` means nothing is written either way. It catches the common breakage — a `README` rewritten so the install line no longer carries the version — at review time instead of at release time.

## Version from the release commit

A build that needs the version it is producing can read it without parsing files:

```yaml
- id: version
  run: echo "version=$(bumpversion show current_version)" >> "$GITHUB_OUTPUT"
```

Downstream steps then use `${{ steps.version.outputs.version }}`.
