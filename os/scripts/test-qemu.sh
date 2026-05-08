#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'USAGE'
Usage: bash os/scripts/test-qemu.sh <path-to-iso>

Boots a LotusOS ISO in QEMU with conservative defaults.
USAGE
}

log() {
  printf '[lotusos:qemu] %s\n' "$*"
}

fail() {
  printf '[lotusos:qemu] ERROR: %s\n' "$*" >&2
  exit 1
}

if [[ "${1:-}" == "-h" || "${1:-}" == "--help" ]]; then
  usage
  exit 0
fi

iso_path="${1:-}"
[[ -n "$iso_path" ]] || fail "Missing ISO path."
[[ -f "$iso_path" ]] || fail "ISO not found: $iso_path"

if ! command -v qemu-system-x86_64 >/dev/null 2>&1; then
  cat >&2 <<'HELP'
[lotusos:qemu] ERROR: qemu-system-x86_64 is missing.

Install QEMU on Debian/Ubuntu/WSL:

  sudo apt update
  sudo apt install qemu-system-x86
HELP
  exit 1
fi

log "Booting ISO: $iso_path"
log "Close the QEMU window to stop the test."

qemu_args=(
  -m 4096 \
  -smp 2 \
  -cdrom "$iso_path" \
  -boot d \
  -vga virtio \
  -display default \
  -netdev user,id=net0 \
  -device virtio-net-pci,netdev=net0
)

if [[ -e /dev/kvm && -r /dev/kvm && -w /dev/kvm ]]; then
  log "Using KVM acceleration."
  qemu_args+=(-enable-kvm -cpu host)
else
  log "KVM is not available; using QEMU software emulation."
  qemu_args+=(-accel tcg)
fi

qemu-system-x86_64 "${qemu_args[@]}"
