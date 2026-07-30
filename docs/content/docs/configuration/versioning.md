---
title: Version scheme
weight: 3
---

# Version scheme

Two keys define what a version is:

- **`parse`** — a regex whose **named capture groups become the version components**.
- **`serialize`** — one or more patterns that turn components back into a string.

The defaults describe SemVer:

```toml
parse = '(?P<major>\d+)\.(?P<minor>\d+)\.(?P<patch>\d+)'
serialize = ["{major}.{minor}.{patch}"]
```

Because `parse` defines the components, changing it changes what you can bump. There is no fixed list of component names — `bumpversion release` works as soon as `parse` has a group called `release`.

## Bumping and resetting

Bumping a component increments it and **resets every component below it**, where "below" means later in the `parse` pattern. That is why `1.4.2` becomes `2.0.0` and not `2.4.2`:

{{< terminal name="show-bump" >}}

## Optional components

A component is optional when it has an `optional_value`. When it holds that value, any `serialize` pattern mentioning it is skipped, and the next pattern is tried. That is how a pre-release ladder collapses back to a plain release.

{{< example path="pre-release/.bumpversion.toml" >}}

`serialize` patterns are tried **in order**, and the first one whose components are all present wins. So put the most specific pattern first: with the pre-release components present, `1.2.0-alpha.1` serializes with the first pattern; once `pre_label` reaches `final` — its `optional_value` — the first pattern no longer applies and the version serializes as plain `1.2.0`.

Walking that ladder:

{{< terminal name="pre-release" >}}

Read the three runs:

- **`pre_n`** advances the iteration within a stage: `alpha.1` to `alpha.2`.
- **`pre_label`** advances the stage and resets the counter below it: `alpha.1` to `beta.0`.
- **`minor`** bumps a component above the pre-release group, so everything below resets — including `pre_label`, which returns to the first of its `values`.

## Component settings

`[tool.bumpversion.parts.<name>]` configures one component. The name must match a capture group in `parse`:

| Key | Type | Meaning |
| --- | --- | --- |
| `values` | list of strings | The allowed values, in order. Without it the component is numeric |
| `optional_value` | string | The value that may be omitted from the serialized version |
| `independent` | bool | The component does not reset when a higher component is bumped |
| `first_value` | string | The value a reset goes to |
| `always_increment` | bool | Increment the component on every bump |
| `depends_on` | string | The component this one resets with |

```toml
[tool.bumpversion.parts.pre_label]
values = ["alpha", "beta", "rc", "final"]
optional_value = "final"
```

A component with `values` is bumped by stepping to the **next entry in the list**, and bumping past the last one is an error rather than a wrap-around. A numeric component is incremented and resets to `0`.

`independent = true` is for a counter that should survive unrelated bumps — a build number that only ever counts up. A capture group whose name starts with `$` is independent automatically.

### Resetting to something other than the first value

By default a reset goes to the first entry of `values`, or to `0` for a numeric component. `first_value` overrides that, and it is what makes a pre-release ladder collapse properly. Without it, a `patch` bump on `1.2.0-alpha.1` gives `1.2.1-alpha.0` — reopening a pre-release nobody asked for. Pointing `first_value` at the optional value fixes it:

```toml
[tool.bumpversion.parts.pre_label]
values = ["alpha", "beta", "rc", "final"]
optional_value = "final"
first_value = "final"
```

Now `patch` on `1.2.0-alpha.1` gives a plain `1.2.1`, and you step back onto the ladder deliberately by bumping `pre_label`.

> [!NOTE]
> `calver_format` is accepted but CalVer formatting is not implemented. Use `{now:%Y%m%d}` in a `serialize` pattern for a date-stamped version.

## Overriding the version directly

Some releases do not follow from a component bump. `--new-version` sets the target explicitly and skips component selection entirely:

```bash
bumpversion --new-version 2.0.0-rc.1
```

`--current-version` overrides the version to start from, for the case where the config file has drifted from reality.

## Per-file schemes

`parse` and `serialize` are also valid inside a `[[files]]` entry, for a file that spells the version differently — a Debian changelog, or a header that carries only `major.minor`:

```toml
[[tool.bumpversion.files]]
filename = "docs/conf.py"
serialize = ["{major}.{minor}"]
```

Inside that entry, `{current_version}` and `{new_version}` render using **that file's** `serialize` patterns, so the search and replacement match what the file actually contains.
