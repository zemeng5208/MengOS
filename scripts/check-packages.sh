#!/usr/bin/env bash
# 作用：检查软件包清单里的每个包在 Debian trixie 中是否存在
set -euo pipefail
LIST_DIR="iso/config/package-lists"
missing=0
total=0
for f in "$LIST_DIR"/*.list.chroot; do
  while IFS= read -r line; do
    pkg="$(echo "$line" | sed 's/#.*//' | xargs)"
    [ -z "$pkg" ] && continue
    total=$((total+1))
    cand="$(apt-cache policy "$pkg" | awk '/Candidate:/ {print $2}')"
    if [ -z "$cand" ] || [ "$cand" = "(none)" ]; then
      echo "MISSING  $pkg"
      missing=$((missing+1))
    else
      echo "OK       $pkg  $cand"
    fi
  done < "$f"
done
echo "总计 $total 个包，缺失 $missing 个"
[ "$missing" -eq 0 ]
