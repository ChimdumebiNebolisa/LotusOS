#!/usr/bin/env bash
set -euo pipefail

iso_path="${1:-artifacts/lotusos-amd64.iso}"
repo_root="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." && pwd)"
iso_file="$repo_root/$iso_path"
work_dir="/tmp/lotusos-iso-verify"
out_file="$repo_root/artifacts/vm-verification/phase5d-iso-shell-paths.txt"

[[ -f "$iso_file" ]] || { echo "Missing ISO: $iso_file" >&2; exit 1; }

rm -rf -- "$work_dir"
mkdir -p "$work_dir"
cd "$work_dir"

xorriso -osirrox on -indev "$iso_file" -extract /live/filesystem.squashfs filesystem.squashfs

{
  printf '=== filesystem paths ===\n'
  unsquashfs -ll filesystem.squashfs | awk '
    /squashfs-root\/opt\/lotus-shell\/lotus-shell$/ ||
    /squashfs-root\/usr\/share\/applications\/lotus-shell\.desktop$/ ||
    /squashfs-root\/etc\/xdg\/autostart\/lotus-shell\.desktop$/ ||
    /squashfs-root\/usr\/local\/bin\/lotus-shell-launch$/ ||
    /squashfs-root\/usr\/share\/applications\/calamares-install-debian\.desktop$/ ||
    /squashfs-root\/etc\/os-release$/ ||
    /squashfs-root\/usr\/share\/applications\/lotus-shell\.desktop$/
  '
} > "$out_file"

echo "Wrote $out_file"
