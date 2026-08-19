---
title: Commands
weight: 5
bookCollapseSection: true
---

# Commands

`bumpversion` has commands for applying and finalizing a bump, plus two that only report.

| Command | Effect |
| --- | --- |
| `bumpversion major` / `minor` / `patch` | Bump that component |
| `bumpversion bump <component>` | Bump any component, including one you defined |
| `bumpversion finalize` | Commit and tag a bump already applied to the working tree |
| `bumpversion show [<variable>...]` | Print resolved config and repository state |
| `bumpversion show-bump <component>` | Print what the next version would be |

- **[Bumping]({{< relref "bump.md" >}})** — the bump commands, `--dry-run`, and how to read the verbose report.
- **[Inspecting]({{< relref "show.md" >}})** — `show` and `show-bump`.
- **[CLI reference]({{< relref "cli-reference.md" >}})** — every flag, the verbosity levels, and the exit codes.

The full help text:

{{< terminal name="help" >}}

> [!NOTE]
> Every flag is global — it is accepted at the top level and on every subcommand, so `bumpversion --dry-run patch` and `bumpversion patch --dry-run` are the same command.
