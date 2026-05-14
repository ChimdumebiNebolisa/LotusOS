#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'USAGE'
Usage: bash os/scripts/clean.sh [--dry-run]

Removes generated LotusOS build artifacts inside this repository only.
USAGE
}

log() {
  printf '[lotusos:clean] %s\n' "$*"
}

fail() {
  printf '[lotusos:clean] ERROR: %s\n' "$*" >&2
  exit 1
}

dry_run=false
for arg in "$@"; do
  case "$arg" in
    --dry-run)
      dry_run=true
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      fail "Unknown argument: $arg"
      ;;
  esac
done

script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd -- "$script_dir/../.." && pwd)"
live_build_dir="$repo_root/os/live-build"

[[ -d "$repo_root/.git" ]] || fail "Run from inside the LotusOS repository."

paths=(
  "$repo_root/artifacts"
  "$live_build_dir/.build"
  "$live_build_dir/cache"
  "$live_build_dir/chroot"
  "$live_build_dir/binary"
  "$live_build_dir/local"
  "$live_build_dir/tmp"
  "$live_build_dir/config/binary"
  "$live_build_dir/config/bootstrap"
  "$live_build_dir/config/chroot"
  "$live_build_dir/config/common"
  "$live_build_dir/config/source"
  "$live_build_dir/lotusos-amd64.iso"
  "$live_build_dir/lotusos-amd64.contents"
  "$live_build_dir/lotusos-amd64.files"
  "$live_build_dir/lotusos-amd64.packages"
)

shopt -s nullglob
for generated in "$live_build_dir"/*.iso "$live_build_dir"/*.contents "$live_build_dir"/*.files "$live_build_dir"/*.packages; do
  paths+=("$generated")
done
shopt -u nullglob

for path in "${paths[@]}"; do
  case "$path" in
    "$repo_root"/*)
      ;;
    *)
      fail "Refusing to clean path outside repository: $path"
      ;;
  esac

  if [[ "$dry_run" == true ]]; then
    log "Would remove: $path"
  elif [[ -e "$path" ]]; then
    rm -rf -- "$path"
    log "Removed: $path"
  else
    log "Not present: $path"
  fi
done
