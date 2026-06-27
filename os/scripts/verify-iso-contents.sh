#!/usr/bin/env bash
set -euo pipefail

iso_path="${1:-artifacts/lotusos-amd64.iso}"
repo_root="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." && pwd)"
iso_file="$repo_root/$iso_path"
timestamp="$(date -u +%Y%m%dT%H%M%SZ)"
work_dir="$(mktemp -d /tmp/lotusos-iso-verify.XXXXXX)"
out_arg="${2:-}"

if [[ -n "$out_arg" ]]; then
  case "$out_arg" in
    /*)
      out_file="$out_arg"
      ;;
    *)
      out_file="$repo_root/$out_arg"
      ;;
  esac
else
  out_file="$repo_root/artifacts/verification/iso-contents-$timestamp.txt"
fi

trap 'rm -rf -- "$work_dir"' EXIT

[[ -f "$iso_file" ]] || { echo "Missing ISO: $iso_file" >&2; exit 1; }

mkdir -p "$work_dir" "$(dirname -- "$out_file")"
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
