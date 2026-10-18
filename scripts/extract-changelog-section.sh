#!/usr/bin/env bash
# Prints the CHANGELOG.md section of one version (the lines between its "## <version>" heading
# and the next "## " heading, without the heading itself). Fails when the version has no section,
# so a release cannot be published without release notes.
# Usage: bash scripts/extract-changelog-section.sh 0.2.0

set -euo pipefail

version="${1:?usage: extract-changelog-section.sh <version>}"
repository_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
changelog_path="$repository_root/CHANGELOG.md"

section="$(awk -v version="$version" '
  /^## / {
    if (in_section) { exit }
    heading_version = $2
    if (heading_version == version) { in_section = 1; next }
  }
  in_section { print }
' "$changelog_path")"

if [[ -z "${section//[[:space:]]/}" ]]; then
  echo "CHANGELOG.md has no section for version $version." >&2
  exit 1
fi

printf '%s\n' "$section"
