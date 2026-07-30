#!/usr/bin/env bash
# Regenerate the documentation terminal snippets under docs/assets/terminals/.
#
# Each snippet is the real, colored output of `bumpversion` run against one of the
# committed example projects in docs/examples/, converted from ANSI to HTML with
# `terminal-to-html` (https://github.com/buildkite/terminal-to-html), which mise
# provides via its github backend.
#
# Those same example files are inlined into the pages by the `example` shortcode,
# so a page shows the config and the output that config actually produces. Change
# an example (or the code behind it) and `task docs:build` regenerates the snippet
# beneath it — a stale example cannot survive a rebuild. That is the whole point;
# the CI docs job regenerates rather than trusting the committed HTML.
#
# Reproducibility notes:
#   * Every run happens in a throwaway, freshly `git init`ed copy of the example,
#     so it is its own repo root, starts from a known commit, and cannot touch
#     this repository — which matters more than usual here, because bumpversion's
#     own .bumpversion.toml has commit and tag enabled.
#   * The temp directory's absolute path leaks into the report (it prints the
#     absolute path of every file it rewrites), so it is rewritten back to `.`;
#     otherwise the random mktemp name would churn the snippets on every run.
#   * `--color always` forces ANSI even though the capture is a pipe, not a PTY.
#   * `--dry-run` gates every write, so no snippet depends on a command having
#     side effects, and the [commit]/[tag] sections still render.
set -euo pipefail

repo="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
bv="$repo/target/debug/bumpversion"
examples="$repo/docs/examples"
out="$repo/docs/assets/terminals"

command -v terminal-to-html >/dev/null || {
  echo "terminal-to-html not found — run 'mise install' (provided via the github backend)" >&2
  exit 1
}
# Always (re)build: a guard could pick up a binary older than the report format
# the snippets are supposed to document. A no-op cargo build is cheap.
cargo build -p bumpversion-cli --bin bumpversion --manifest-path "$repo/Cargo.toml"
mkdir -p "$out"

# Run bumpversion once against a throwaway copy of an example, emitting a prompt
# line plus the captured ANSI on stdout. Usage: run <example-dir> <args...>
run() {
  local ex="$1"
  shift
  # `pwd -P` resolves the symlinked temp root (/var → /private/var on macOS) so the
  # path substitution below matches what bumpversion prints after canonicalizing.
  local work
  work="$(cd "$(mktemp -d)" && pwd -P)"
  [[ -n "$work" && -d "$work" ]] || {
    echo "failed to create a scratch directory" >&2
    exit 1
  }
  cp -R "$examples/$ex/." "$work/"
  git init -q -b main "$work"
  git -C "$work" add -A
  git -C "$work" \
    -c user.email=docs@example.com -c user.name=docs -c commit.gpgsign=false \
    commit -qm "initial commit"
  # Tag the initial commit at the example's own current_version, so the scratch
  # repo looks like a project that has already shipped: `show current_tag` and
  # `{current_tag}` resolve, and bumpversion does not warn that the configured
  # version disagrees with the last tag. Read from the example rather than
  # hardcoded, so bumping an example's version needs no edit here.
  local config version=""
  for config in "$work/.bumpversion.toml" "$work/pyproject.toml" "$work/setup.cfg"; do
    [[ -f "$config" ]] || continue
    version="$(sed -n 's/^current_version[[:space:]]*=[[:space:]]*"\{0,1\}\([^"]*\)"\{0,1\}$/\1/p' "$config")"
    [[ -n "$version" ]] && break
  done
  if [[ -n "$version" ]]; then
    git -C "$work" tag "v$version"
  fi

  printf '\033[1;32m$\033[0m bumpversion %s\n\n' "$*"
  # `|| true` keeps a non-zero exit from aborting the script under `set -e`: some
  # snippets deliberately show an error path.
  (cd "$work" && "$bv" --dir . --color always "$@" 2>&1) | sed "s#$work#.#g" || true
  rm -rf "$work"
}

# Convert the ANSI on stdin into $out/<name>.html.
render() {
  terminal-to-html >"$out/$1.html"
  echo "wrote $out/$1.html"
}

# The top-level CLI overview. No repository needed.
{
  printf '\033[1;32m$\033[0m bumpversion --help\n\n'
  "$bv" --help 2>&1
} | render help

# The landing-page hero and the quick start: a patch bump over the simple example.
# -vv adds the per-component breakdown under each version.
run simple --dry-run -vv bump patch | render bump

# What the next version would be, without touching anything.
{
  run simple show-bump patch
  printf '\n'
  run simple show-bump minor
  printf '\n'
  run simple show-bump major
} | render show-bump

# Reading values out of the resolved config and the repository state. One variable
# prints a bare value; two or more print `name=value` lines.
{
  run simple show current_version
  printf '\n'
  run simple show current_version current_tag branch_name
} | render show

# Optional components: `pre_label`/`pre_n` walk the pre-release ladder, while a
# major/minor/patch bump resets it back to the first value.
{
  run pre-release show-bump pre_n
  printf '\n'
  run pre-release show-bump pre_label
  printf '\n'
  run pre-release show-bump minor
} | render pre-release

# One `glob` entry covering every package in the workspace.
run monorepo --dry-run -v bump major | render monorepo

# Hooks, a custom commit message and tag name, and an `additional_files` entry
# that carries a hook-generated CHANGELOG.md into the release commit.
run hooks --dry-run -v bump minor | render hooks

# The config living in pyproject.toml instead of its own file.
run pyproject --dry-run -v bump minor | render pyproject
