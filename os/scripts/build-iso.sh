#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'USAGE'
Usage: bash os/scripts/build-iso.sh [--check]

Builds the LotusOS live ISO with Debian live-build.

Options:
  --check   Validate host dependencies and paths without building.
USAGE
}

log() {
  printf '[lotusos:build] %s\n' "$*"
}

fail() {
  printf '[lotusos:build] ERROR: %s\n' "$*" >&2
  exit 1
}

check_only=false
for arg in "$@"; do
  case "$arg" in
    --check)
      check_only=true
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

if [[ "${BASH_SOURCE[0]}" == */* ]]; then
  script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
else
  script_dir="$(pwd)"
fi

repo_root="$(cd -- "$script_dir/../.." && pwd)"
source_live_build_dir="$repo_root/os/live-build"
build_root="${LOTUSOS_BUILD_ROOT:-/tmp/lotusos-live-build}"
live_build_dir="$build_root/live-build"
artifacts_dir="$repo_root/artifacts"
iso_name="lotusos-amd64.iso"

[[ -d "$repo_root/.git" ]] || fail "Run from inside the LotusOS repository."
[[ -d "$source_live_build_dir/config" ]] || fail "Missing live-build config at $source_live_build_dir/config."

case "$build_root" in
  /tmp/lotusos-*|/var/tmp/lotusos-*)
    ;;
  *)
    fail "Refusing build root outside an expected temporary LotusOS path: $build_root"
    ;;
esac

case "$(uname -s)" in
  Linux)
    ;;
  *)
    fail "live-build must run on Linux or WSL2. Use WSL Ubuntu from Windows."
    ;;
esac

missing=()
for cmd in lb xorriso mksquashfs; do
  if ! command -v "$cmd" >/dev/null 2>&1; then
    missing+=("$cmd")
  fi
done

if ((${#missing[@]} > 0)); then
  printf '[lotusos:build] Missing required command(s): %s\n' "${missing[*]}" >&2
  cat >&2 <<'HELP'
Install the initial build dependencies on Debian/Ubuntu/WSL:

  sudo apt update
  sudo apt install live-build xorriso isolinux syslinux-common squashfs-tools
HELP
  exit 1
fi

log "Repository: $repo_root"
log "Source live-build directory: $source_live_build_dir"
log "Working live-build directory: $live_build_dir"
log "Artifact target: $artifacts_dir/$iso_name"

if [[ "$check_only" == true ]]; then
  log "Dependency check passed. No ISO build started."
  exit 0
fi

if [[ "${EUID:-$(id -u)}" -ne 0 ]]; then
  fail "Building with live-build usually requires root. Re-run with sudo: sudo bash os/scripts/build-iso.sh"
fi

mkdir -p "$artifacts_dir"

log "Preparing native Linux build workspace at $build_root"
rm -rf -- "$build_root"
mkdir -p "$live_build_dir"
cp -a "$source_live_build_dir/auto" "$source_live_build_dir/config" "$source_live_build_dir/hooks" "$live_build_dir/"
rm -f -- \
  "$live_build_dir/config/binary" \
  "$live_build_dir/config/bootstrap" \
  "$live_build_dir/config/chroot" \
  "$live_build_dir/config/common" \
  "$live_build_dir/config/source"

log "Starting live-build."
(
  cd "$live_build_dir"
  lb config
  lb build
)

mapfile -t produced_isos < <(find "$live_build_dir" -maxdepth 1 -type f -name '*.iso' -printf '%T@ %p\n' | sort -nr | awk '{print $2}')
if ((${#produced_isos[@]} == 0)); then
  fail "live-build completed but no ISO was found in $live_build_dir."
fi

source_iso="${produced_isos[0]}"

cp "$source_iso" "$artifacts_dir/$iso_name"
log "ISO copied to $artifacts_dir/$iso_name"
log "Next: bash os/scripts/test-qemu.sh artifacts/$iso_name"
