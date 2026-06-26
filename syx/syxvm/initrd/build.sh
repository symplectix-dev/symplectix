#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
OUT="/data/firecracker/initrd.cpio.gz"

cd "$REPO_ROOT"
bazel build //syx/syxvm/initrd:initrd
install -m644 bazel-bin/syx/syxvm/initrd/initrd.cpio.gz "$OUT"
echo "Done: $(du -sh "$OUT" | cut -f1)  $OUT"
