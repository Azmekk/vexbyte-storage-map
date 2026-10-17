#!/usr/bin/env bash
# Fails when Rust code gates on the operating system outside the places PLAN.md section 16 allows:
# platform/mod.rs files, the MFT module root, src-tauri/src/elevation.rs and Cargo.toml files.
# Runs in CI and locally (Git Bash on Windows): bash scripts/check-platform-cfg.sh

set -euo pipefail

repository_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repository_root"

platform_cfg_pattern='cfg\(\s*((not|any|all)\(\s*)*(windows|unix|target_os|target_family)\b'
allowed_files_pattern='(^|/)platform/mod\.rs$|^crates/disk_scanner/src/mft/mod\.rs$|^src-tauri/src/elevation\.rs$'

files_with_platform_cfg="$(grep -rlE "$platform_cfg_pattern" --include='*.rs' crates src-tauri/src || true)"
violating_files="$(printf '%s\n' "$files_with_platform_cfg" | grep -vE "$allowed_files_pattern" | grep -v '^$' || true)"

if [[ -n "$violating_files" ]]; then
  echo "Operating system cfg found outside the allowed places (see PLAN.md section 16):"
  while IFS= read -r violating_file; do
    grep -nE "$platform_cfg_pattern" "$violating_file" | sed "s|^|$violating_file:|"
  done <<< "$violating_files"
  exit 1
fi

echo "Platform cfg check passed."
