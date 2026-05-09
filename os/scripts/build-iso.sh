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

repair_grub2_eltorito_iso() {
  local grub_img="$live_build_dir/binary/boot/grub/grub_eltorito"
  local cdboot_img="$live_build_dir/chroot/usr/lib/grub/i386-pc/cdboot.img"
  local grub_dir="$live_build_dir/chroot/usr/lib/grub/i386-pc"
  local grub_mkimage="$live_build_dir/chroot/usr/bin/grub-mkimage"
  local linker="$live_build_dir/chroot/lib64/ld-linux-x86-64.so.2"
  local lib_path="$live_build_dir/chroot/lib/x86_64-linux-gnu:$live_build_dir/chroot/usr/lib/x86_64-linux-gnu"
  local core_img
  local grub_size

  [[ -f "$grub_img" ]] || return 0

  grub_size="$(stat -c '%s' "$grub_img")"
  if ((grub_size > 4096)); then
    log "GRUB2 El Torito image already contains a core image ($grub_size bytes)."
    return 0
  fi

  log "Detected GRUB2 El Torito stub without core image ($grub_size bytes)."
  log "Rebuilding GRUB2 El Torito image with explicit /boot/grub prefix."

  [[ -f "$cdboot_img" ]] || fail "Missing GRUB cdboot image: $cdboot_img"
  [[ -d "$grub_dir" ]] || fail "Missing GRUB module directory: $grub_dir"
  [[ -x "$grub_mkimage" ]] || fail "Missing grub-mkimage in live-build chroot: $grub_mkimage"
  [[ -x "$linker" ]] || fail "Missing chroot dynamic linker: $linker"

  core_img="$(mktemp)"
  "$linker" --library-path "$lib_path" "$grub_mkimage" \
    -d "$grub_dir" \
    -O i386-pc \
    -p /boot/grub \
    -o "$core_img" \
    biosdisk iso9660 normal configfile

  cat "$cdboot_img" "$core_img" > "$grub_img"
  rm -f -- "$core_img"

  grub_size="$(stat -c '%s' "$grub_img")"
  if ((grub_size <= 4096)); then
    fail "Rebuilt GRUB2 El Torito image is still too small: $grub_size bytes"
  fi

  log "Rebuilt GRUB2 El Torito image ($grub_size bytes)."
  log "Rebuilding ISO with repaired GRUB2 boot image."

  rm -f -- "$live_build_dir/binary.iso"
  xorriso -as mkisofs \
    -J \
    -l \
    -cache-inodes \
    -allow-multidot \
    -A "LotusOS Live" \
    -publisher "LotusOS Project" \
    -V "LotusOS amd64" \
    -no-emul-boot \
    -boot-load-size 4 \
    -boot-info-table \
    -r \
    -b boot/grub/grub_eltorito \
    -o "$live_build_dir/binary.iso" \
    "$live_build_dir/binary"
}

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

repair_grub2_eltorito_iso

mapfile -t produced_isos < <(find "$live_build_dir" -maxdepth 1 -type f -name '*.iso' -printf '%T@ %p\n' | sort -nr | awk '{print $2}')
if ((${#produced_isos[@]} == 0)); then
  fail "live-build completed but no ISO was found in $live_build_dir."
fi

source_iso="${produced_isos[0]}"

cp "$source_iso" "$artifacts_dir/$iso_name"
log "ISO copied to $artifacts_dir/$iso_name"
log "Next: bash os/scripts/test-qemu.sh artifacts/$iso_name"
