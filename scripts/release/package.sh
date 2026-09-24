#!/usr/bin/env bash
# Archive one built `eplyx` binary as a release artifact containing only that
# binary: eplyx-v<version>-<platform>.tar.gz (or .zip on Windows).
#   scripts/release/package.sh <binary> <platform> <out-dir>
set -euo pipefail
binary=$1 platform=$2 out=$3
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
version=$(sed -n 's/^version = "\(.*\)"/\1/p' "$root/Cargo.toml" | head -n 1)
[ -n "$version" ] || { echo "workspace version not found" >&2; exit 1; }
case "$platform" in
  darwin-arm64 | linux-x86_64) name=eplyx ext=tar.gz ;;
  windows-x86_64) name=eplyx.exe ext=zip ;;
  *) echo "unsupported release platform: $platform" >&2; exit 1 ;;
esac
[ -f "$binary" ] || { echo "missing binary: $binary" >&2; exit 1; }
mkdir -p "$out"
out="$(cd "$out" && pwd)"
stage=$(mktemp -d)
trap 'rm -rf "$stage"' EXIT
cp "$binary" "$stage/$name"
chmod 755 "$stage/$name"
artifact="eplyx-v$version-$platform.$ext"
rm -f "$out/$artifact"
if [ "$ext" = zip ]; then
  (cd "$stage" && if command -v zip >/dev/null 2>&1; then zip -q -X "$out/$artifact" "$name"; else 7z a -tzip -bso0 "$out/$artifact" "$name"; fi)
else
  # No local user/group names, extended attributes or AppleDouble files.
  if tar --version 2>/dev/null | grep -q 'GNU tar'; then
    tar --owner=0 --group=0 --numeric-owner -czf "$out/$artifact" -C "$stage" "$name"
  else
    COPYFILE_DISABLE=1 tar --uid 0 --gid 0 --uname '' --gname '' --no-xattrs --no-mac-metadata -czf "$out/$artifact" -C "$stage" "$name"
  fi
fi
echo "$out/$artifact"
