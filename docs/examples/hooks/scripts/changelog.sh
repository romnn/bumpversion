#!/usr/bin/env sh
# Turn the "Unreleased" heading into a released one. Invoked as a pre_commit_hook,
# so CHANGELOG.md is listed in `additional_files` to reach the release commit.
set -eu
sed -i.bak "s/^## Unreleased$/## $1/" CHANGELOG.md
rm -f CHANGELOG.md.bak
