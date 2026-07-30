---
title: Inspecting
weight: 2
---

# Inspecting

Two commands report without changing anything. Both skip the dirty-tree check, so they work in a repository with uncommitted work.

## `show-bump`

Computes the next version for a component and prints it:

```bash
bumpversion show-bump patch
```

{{< terminal name="show-bump" >}}

The output is two `name=value` lines, which is easy to consume from a shell:

```bash
eval "$(bumpversion show-bump minor)"
echo "releasing $new_version"
```

It answers "what would this bump produce" without the noise of a full dry-run report — useful when you are working out a [pre-release ladder]({{< relref "../configuration/versioning.md" >}}#optional-components) and want to see where each component lands.

## `show`

Prints resolved configuration and repository state:

{{< terminal name="show" >}}

The output format depends on how many variables you ask for:

- **One variable** — the bare value, ready to capture in a shell variable.
- **Two or more** — one `name=value` line per variable.

```bash
version=$(bumpversion show current_version)
```

Most [placeholders]({{< relref "../configuration/reference.md" >}}#placeholders) can be shown: `current_version`, `current_tag`, `branch_name`, `commit_sha`, `distance_to_latest_tag`, `dirty`, `repository_root`, `now`, `utcnow`, and the per-component values. There is also a `files` variable listing the resolved file set, one path per line — the quickest way to check that a `glob` matches what you expected.

`new_version` is an exception: no bump is in progress during a `show`, so it resolves to nothing. Use [`show-bump`](#show-bump) for that.

An unknown name produces a warning and is skipped; the command still exits `0`, so a typo will not fail a script.

```text
WARN bumpversion::common: variable tag_name not found in context
```

> [!NOTE]
> `show` reports *context values*, not config keys. `bumpversion show tag_name` does not work, because `tag_name` is a template you configure rather than a value in the context.

## Debugging a configuration

When a bump does not do what you expect, the order that usually finds it:

1. `bumpversion show current_version` — is the version being read from the file you think?
2. `bumpversion show files` — does the file set match what you intended, especially with globs?
3. `bumpversion show-bump <component>` — does the version arithmetic produce the right number?
4. `bumpversion --dry-run -vv <component>` — does each file's `search` template resolve to a string the file actually contains?

Step 4 is the one that catches most problems: the report prints each rendered template beside its concrete value, so a `search` that matched nothing stands out immediately.
