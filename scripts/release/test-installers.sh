#!/usr/bin/env bash
# Installer tests against crafted local (file://) releases. No network.
#   scripts/release/test-installers.sh <unix-release-archive>
# The archive must hold a real `eplyx` binary for this machine.
set -euo pipefail
archive=$(cd "$(dirname "$1")" && pwd)/$(basename "$1")
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
installer="$root/scripts/install/install.sh"
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
failures=0
sha() { if command -v sha256sum >/dev/null; then sha256sum "$1" | cut -d ' ' -f 1; else shasum -a 256 "$1" | cut -d ' ' -f 1; fi; }
case "$(uname -s)-$(uname -m)" in Darwin-arm64) platform=darwin-arm64 ;; Linux-x86_64) platform=linux-x86_64 ;; *) echo "unsupported test host"; exit 1 ;; esac
version=$(basename "$archive" | sed -n 's/^eplyx-\(v[^-]*\)-.*/\1/p')

# release <name> <archive-file> [extra SHA256SUMS lines...]: a local release dir.
release() {
  local dir="$work/$1" file=$2; shift 2
  mkdir -p "$dir"
  cp "$file" "$dir/eplyx-$version-$platform.tar.gz"
  { echo "$(sha "$dir/eplyx-$version-$platform.tar.gz")  eplyx-$version-$platform.tar.gz"; printf '%s\n' "$@"; } > "$dir/SHA256SUMS"
  echo "$dir"
}
# crafted <name> <builder>: an archive assembled by a shell snippet in $stage.
crafted() {
  local stage="$work/stage-$1"; mkdir -p "$stage"
  (cd "$stage" && eval "$2")
  echo "$work/crafted-$1.tar.gz"
}
# run <name> <expect: ok|fail> <message-substring> [env...] : install into a fresh HOME.
run() {
  local name=$1 expect=$2 needle=$3; shift 3
  local home="$work/home-$name" tmpdir="$work/tmp-$name"
  mkdir -p "$home" "$tmpdir"
  set +e
  output=$(env -i PATH="/usr/bin:/bin:/usr/sbin:/sbin" HOME="$home" TMPDIR="$tmpdir" SHELL=/bin/zsh "$@" sh "$installer" 2>&1)
  code=$?
  set -e
  local ok=1
  if [ "$expect" = ok ] && [ "$code" != 0 ]; then ok=0; fi
  if [ "$expect" = fail ] && [ "$code" = 0 ]; then ok=0; fi
  grep -qF -- "$needle" <<<"$output" || ok=0
  if [ -n "$(ls -A "$tmpdir")" ]; then ok=0; output="$output
(temporary files left behind)"; fi
  if [ "$expect" = fail ] && [ -e "$home/.local/bin/eplyx" ]; then ok=0; output="$output
(binary installed despite failure)"; fi
  if [ "$ok" = 1 ]; then echo "ok   $name"; else echo "FAIL $name (exit $code)"; echo "$output" | sed 's/^/     /'; failures=$((failures + 1)); fi
}

good=$(release good "$archive" \
  "$(printf '%064d' 1)  eplyx-$version-windows-x86_64.zip" \
  "$(printf '%064d' 2)  eplyx-$version-linux-arm64.tar.gz" \
  "$(printf '%064d' 3)  install.sh")
run default-install ok "Installed eplyx" EPLYX_DOWNLOAD_BASE="file://$good"
[ -x "$work/home-default-install/.local/bin/eplyx" ] && echo "ok   default dir is \$HOME/.local/bin" || { echo "FAIL default dir"; failures=$((failures + 1)); }
[ "$(cd "$work/home-default-install" && find . -type f | sort)" = "./.local/bin/eplyx" ] && echo "ok   only the binary is written; no profile, config or credential files" || { echo "FAIL extra files written"; failures=$((failures + 1)); }
run path-guidance ok "export PATH=\"$work/home-path-guidance/.local/bin:\$PATH\"" EPLYX_DOWNLOAD_BASE="file://$good"
run profile-hint ok "add that line to ~/.zshrc" EPLYX_DOWNLOAD_BASE="file://$good"
mkdir -p "$work/custom dir"
run custom-dir ok "Installed eplyx" EPLYX_DOWNLOAD_BASE="file://$good" EPLYX_INSTALL_DIR="$work/custom dir"
[ -x "$work/custom dir/eplyx" ] && echo "ok   EPLYX_INSTALL_DIR with a space" || { echo "FAIL custom dir"; failures=$((failures + 1)); }
run pinned-version ok "Installed eplyx" EPLYX_DOWNLOAD_BASE="file://$good" EPLYX_VERSION="$version"
run relative-dir fail "must be an absolute path" EPLYX_DOWNLOAD_BASE="file://$good" EPLYX_INSTALL_DIR=relative/bin
run http-base fail "must be an https:// URL" EPLYX_DOWNLOAD_BASE="http://example.com/release"
run bad-version fail "must look like v0.1.0" EPLYX_DOWNLOAD_BASE="file://$good" EPLYX_VERSION='v1.*'
run unsupported fail "unsupported platform 'freebsd-x86_64'" EPLYX_DOWNLOAD_BASE="file://$good" EPLYX_PLATFORM=freebsd-x86_64
run unsupported-arm fail "unsupported platform 'linux-arm64'" EPLYX_DOWNLOAD_BASE="file://$good" EPLYX_PLATFORM=linux-arm64

two=$(release two-versions "$archive" "$(printf '%064d' 4)  eplyx-v9.9.9-$platform.tar.gz")
run ambiguous fail "exactly one archive for $platform (found 2)" EPLYX_DOWNLOAD_BASE="file://$two"
run ambiguous-pinned ok "Installed eplyx" EPLYX_DOWNLOAD_BASE="file://$two" EPLYX_VERSION="$version"

mismatch="$work/mismatch"; mkdir -p "$mismatch"; cp "$archive" "$mismatch/eplyx-$version-$platform.tar.gz"
echo "$(printf '%064d' 0)  eplyx-$version-$platform.tar.gz" > "$mismatch/SHA256SUMS"
run sha-mismatch fail "checksum mismatch" EPLYX_DOWNLOAD_BASE="file://$mismatch"
missing="$work/missing"; mkdir -p "$missing"; echo "$(printf '%064d' 5)  eplyx-$version-other-os.tar.gz" > "$missing/SHA256SUMS"
run no-artifact fail "(found 0)" EPLYX_DOWNLOAD_BASE="file://$missing"
malformed="$work/malformed"; mkdir -p "$malformed"; echo "abc  eplyx-$version-$platform.tar.gz" > "$malformed/SHA256SUMS"
run malformed-sums fail "(found 0)" EPLYX_DOWNLOAD_BASE="file://$malformed"

# -P keeps the "../" member name on both GNU and BSD tar.
traversal=$(crafted traversal 'printf x > evil; mkdir inner; cd inner; tar -P -czf "'"$work"'/crafted-traversal.tar.gz" ../evil')
run traversal fail "unexpected archive layout" EPLYX_DOWNLOAD_BASE="file://$(release traversal "$traversal")"
extra=$(crafted extra 'cp "'"$archive"'" a.tgz; tar -xzf a.tgz; rm a.tgz; printf x > README; tar -czf "'"$work"'/crafted-extra.tar.gz" eplyx README')
run extra-member fail "unexpected archive layout" EPLYX_DOWNLOAD_BASE="file://$(release extra "$extra")"
dotted=$(crafted dotted 'cp "'"$archive"'" a.tgz; tar -xzf a.tgz; rm a.tgz; tar -czf "'"$work"'/crafted-dotted.tar.gz" ./eplyx')
run dot-slash-member fail "unexpected archive layout" EPLYX_DOWNLOAD_BASE="file://$(release dotted "$dotted")"
noblob=$(crafted noblob 'printf x > README; tar -czf "'"$work"'/crafted-noblob.tar.gz" README')
run missing-binary fail "unexpected archive layout" EPLYX_DOWNLOAD_BASE="file://$(release noblob "$noblob")"
link=$(crafted link 'ln -s /bin/sh eplyx; tar -czf "'"$work"'/crafted-link.tar.gz" eplyx')
run symlink-member fail "not a regular file" EPLYX_DOWNLOAD_BASE="file://$(release link "$link")"
notar=$(crafted notar 'printf "not an archive" > "'"$work"'/crafted-notar.tar.gz"')
run not-an-archive fail "not a readable tar.gz" EPLYX_DOWNLOAD_BASE="file://$(release notar "$notar")"

echo
if [ "$failures" = 0 ]; then echo "installer tests passed"; else echo "$failures installer tests failed"; exit 1; fi
