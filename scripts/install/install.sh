#!/bin/sh
# Install the Eplyx CLI on macOS or Linux.
#
#   curl -fsSL https://github.com/thomasdevving/eplyx-stock-transition-assurance/releases/latest/download/install.sh | sh
#
# Downloads one release archive and the release SHA256SUMS over HTTPS, verifies
# the archive's SHA-256, requires the archive to contain exactly the `eplyx`
# binary, and copies that binary into a user-owned directory. It never edits
# shell profiles, never needs sudo, sends nothing about you and stores no
# credentials. Everything it downloads is removed when it exits.
#
# Environment (all optional):
#   EPLYX_VERSION        release tag to install, e.g. v0.1.0 (default: latest)
#   EPLYX_INSTALL_DIR    absolute install directory (default: $HOME/.local/bin)
#   EPLYX_DOWNLOAD_BASE  https:// (or file:// for an offline mirror) directory
#                        holding the release assets
#   EPLYX_PLATFORM       override detection: darwin-arm64 or linux-x86_64
set -eu

REPO="thomasdevving/eplyx-stock-transition-assurance"
SUPPORTED="darwin-arm64 linux-x86_64"

say() { printf '%s\n' "$*"; }
die() { printf 'eplyx install: %s\n' "$*" >&2; exit 1; }

detect_platform() {
  if [ -n "${EPLYX_PLATFORM:-}" ]; then
    printf '%s' "$EPLYX_PLATFORM"
    return
  fi
  os=$(uname -s 2>/dev/null || echo unknown)
  arch=$(uname -m 2>/dev/null || echo unknown)
  case "$os" in
    Darwin) os=darwin ;;
    Linux) os=linux ;;
    *) os=$(printf '%s' "$os" | tr '[:upper:]' '[:lower:]') ;;
  esac
  case "$arch" in
    arm64 | aarch64) arch=arm64 ;;
    x86_64 | amd64) arch=x86_64 ;;
  esac
  # An x86_64 shell under Rosetta on Apple Silicon still gets the native build.
  if [ "$os" = darwin ] && [ "$arch" = x86_64 ] && [ "$(sysctl -n sysctl.proc_translated 2>/dev/null || echo 0)" = 1 ]; then
    arch=arm64
  fi
  printf '%s-%s' "$os" "$arch"
}

supported() {
  for candidate in $SUPPORTED; do
    [ "$candidate" = "$1" ] && return 0
  done
  return 1
}

download() { # url destination
  case "$1" in
    https://*)
      if command -v curl >/dev/null 2>&1; then
        curl --proto '=https' --tlsv1.2 -fsSL --retry 3 -o "$2" "$1" || die "download failed: $1"
      elif command -v wget >/dev/null 2>&1; then
        wget -q --https-only -O "$2" "$1" || die "download failed: $1"
      else
        die "curl or wget is required"
      fi
      ;;
    file://*)
      source_path=${1#file://}
      [ -f "$source_path" ] || die "missing local release asset: $source_path"
      cp "$source_path" "$2"
      ;;
    *) die "refusing non-HTTPS download: $1" ;;
  esac
}

sha256_of() {
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "$1" | cut -d ' ' -f 1
  elif command -v shasum >/dev/null 2>&1; then
    shasum -a 256 "$1" | cut -d ' ' -f 1
  else
    die "sha256sum or shasum is required to verify the download"
  fi
}

platform=$(detect_platform)
supported "$platform" || die "unsupported platform '$platform'. Supported: $SUPPORTED (Windows: use install.ps1). See the README for building from source."

if [ -n "${EPLYX_VERSION:-}" ]; then
  printf '%s' "$EPLYX_VERSION" | grep -Eq '^v[0-9]+\.[0-9]+\.[0-9]+(-[0-9A-Za-z.]+)?$' \
    || die "EPLYX_VERSION must look like v0.1.0"
fi
if [ -n "${EPLYX_DOWNLOAD_BASE:-}" ]; then
  base=${EPLYX_DOWNLOAD_BASE%/}
elif [ -n "${EPLYX_VERSION:-}" ]; then
  base="https://github.com/$REPO/releases/download/$EPLYX_VERSION"
else
  base="https://github.com/$REPO/releases/latest/download"
fi
case "$base" in
  https://* | file:///*) ;;
  *) die "EPLYX_DOWNLOAD_BASE must be an https:// URL or an absolute file:// path" ;;
esac

install_dir=${EPLYX_INSTALL_DIR:-${HOME:?HOME is not set}/.local/bin}
case "$install_dir" in
  /*) ;;
  *) die "EPLYX_INSTALL_DIR must be an absolute path" ;;
esac

tmp=$(mktemp -d 2>/dev/null || mktemp -d -t eplyx-install)
trap 'rm -rf "$tmp"' EXIT
trap 'rm -rf "$tmp"; exit 130' INT TERM

download "$base/SHA256SUMS" "$tmp/SHA256SUMS"

# Exactly one checksum line must name this platform's archive; a pinned
# version must match that archive's version.
# The pattern travels through ENVIRON because `awk -v` rewrites backslashes.
version_pattern='v[0-9]+[.][0-9]+[.][0-9]+(-[0-9A-Za-z.]+)?'
if [ -n "${EPLYX_VERSION:-}" ]; then
  version_pattern=$(printf '%s' "$EPLYX_VERSION" | sed 's/[.]/[.]/g')
fi
EPLYX_ARCHIVE_PATTERN="^eplyx-${version_pattern}-${platform}[.]tar[.]gz\$"
export EPLYX_ARCHIVE_PATTERN
matches=$(awk 'NF == 2 && $1 ~ /^[0-9a-f]+$/ && length($1) == 64 && $2 ~ ENVIRON["EPLYX_ARCHIVE_PATTERN"] { print }' "$tmp/SHA256SUMS")
count=$(printf '%s' "$matches" | grep -c . || true)
[ "$count" = 1 ] || die "SHA256SUMS must list exactly one archive for $platform (found $count)"
expected=$(printf '%s' "$matches" | cut -d ' ' -f 1)
archive=$(printf '%s' "$matches" | awk '{ print $2 }')

say "Downloading $archive"
download "$base/$archive" "$tmp/$archive"
actual=$(sha256_of "$tmp/$archive")
[ "$actual" = "$expected" ] || die "checksum mismatch for $archive (expected $expected, got $actual); nothing was installed"
say "Verified SHA-256 $actual"

# The archive must contain exactly one member, the binary itself. This rejects
# paths, traversal, links to elsewhere and any extra file before extraction.
members=$(tar -tzf "$tmp/$archive") || die "archive is not a readable tar.gz"
[ "$members" = "eplyx" ] || die "unexpected archive layout; expected only 'eplyx'"
mkdir "$tmp/extract"
tar -xzf "$tmp/$archive" -C "$tmp/extract" eplyx || die "could not extract eplyx"
binary="$tmp/extract/eplyx"
[ -f "$binary" ] && [ ! -L "$binary" ] || die "archive member 'eplyx' is not a regular file"
chmod 755 "$binary"
"$binary" --version >"$tmp/version" 2>&1 || die "the verified binary does not run on this system"
installed_version=$(head -n 1 "$tmp/version")

mkdir -p "$install_dir"
cp "$binary" "$install_dir/.eplyx.$$"
mv -f "$install_dir/.eplyx.$$" "$install_dir/eplyx"

say "Installed $installed_version to $install_dir/eplyx"
case ":${PATH:-}:" in
  *":$install_dir:"*) ;;
  *)
    profile="your shell profile"
    case "${SHELL:-}" in
      */zsh) profile="~/.zshrc" ;;
      */bash) profile="~/.bashrc" ;;
      */fish) profile="~/.config/fish/config.fish" ;;
    esac
    say ""
    say "$install_dir is not on your PATH. For this shell run:"
    say "  export PATH=\"$install_dir:\$PATH\""
    say "To keep it, add that line to $profile. This installer did not change any profile."
    ;;
esac
say ""
say "Next, in your Solana project:"
say "  eplyx init && eplyx doctor"
say "Uninstall: rm \"$install_dir/eplyx\" (project data stays in each project's .eplyx/)."
