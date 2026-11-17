#!/usr/bin/env bash
# 作用：在 Debian trixie 中构建 MengOS ISO，输出到 dist/
set -euo pipefail
VERSION="${MENGOS_VERSION:-0.0.1-dev}"
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT/iso"
lb clean --purge || true
lb config
lb build
mkdir -p "$ROOT/dist"
ISO_SRC="$(ls -1 live-image-amd64*.iso | head -n1)"
ISO_DST="$ROOT/dist/MengOS-${VERSION}-amd64.iso"
mv "$ISO_SRC" "$ISO_DST"
cp live-image-amd64.packages "$ROOT/dist/MengOS-${VERSION}-amd64.packages.txt" || true
cd "$ROOT/dist"
sha256sum "$(basename "$ISO_DST")" > "$(basename "$ISO_DST").sha256"
ls -lh
cat ./*.sha256
