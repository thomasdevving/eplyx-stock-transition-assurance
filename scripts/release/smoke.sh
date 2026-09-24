#!/usr/bin/env bash
# Smoke-test the exact binary inside a release archive. The binary is taken
# from the archive, never from target/, and runs outside the repository.
#   scripts/release/smoke.sh <archive>
set -euo pipefail
archive=$(cd "$(dirname "$1")" && pwd)/$(basename "$1")
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
work=$(mktemp -d)
dashboard_pid=""
cleanup() { [ -n "$dashboard_pid" ] && kill "$dashboard_pid" 2>/dev/null || true; rm -rf "$work"; }
trap cleanup EXIT
fail() { echo "smoke: FAIL: $*" >&2; exit 1; }
pass() { echo "smoke: ok: $*"; }
sha() { if command -v sha256sum >/dev/null; then sha256sum "$1" | cut -d ' ' -f 1; else shasum -a 256 "$1" | cut -d ' ' -f 1; fi; }
archive_sha=$(sha "$archive")

mkdir "$work/bin"
case "$archive" in
  *.zip)
    if command -v unzip >/dev/null; then unzip -q "$archive" -d "$work/bin"; else 7z x -bso0 "-o$work/bin" "$archive"; fi
    bin="$work/bin/eplyx.exe" members=$(ls -A "$work/bin") expected=eplyx.exe ;;
  *)
    members=$(tar -tzf "$archive") expected=eplyx
    tar -xzf "$archive" -C "$work/bin"; bin="$work/bin/eplyx" ;;
esac
[ "$members" = "$expected" ] || fail "archive must contain only $expected, found: $members"
pass "archive contains only $expected"

# Version: no project, config, store or network.
mkdir "$work/empty" && cd "$work/empty"
version_line=$(HTTPS_PROXY=http://127.0.0.1:9 HTTP_PROXY=http://127.0.0.1:9 "$bin" --version | head -n 1)
[[ "$version_line" =~ ^eplyx\ [0-9]+\.[0-9]+\.[0-9]+ ]] || fail "unexpected --version: $version_line"
"$bin" --version | grep -q '^commit [0-9a-f]\{12\}$' || fail "--version lacks a build commit"
"$bin" version --json | grep -q '"counterexample_search"' || fail "version --json lacks engine schemas"
[ -z "$(ls -A "$work/empty")" ] || fail "--version wrote files"
pass "$version_line"

# A clean project: init, local store and an offline doctor.
mkdir "$work/clean" && cd "$work/clean"
"$bin" init > /dev/null
[ -f eplyx.toml ] && [ -f .eplyx/project.json ] || fail "init did not create eplyx.toml and .eplyx/"
[ "$("$bin" runs --json | tr -d '[:space:]')" = "[]" ] || fail "runs is not empty in a clean project"
set +e; doctor=$("$bin" doctor --offline); code=$?; set -e
[ "$code" = 2 ] || fail "doctor --offline without a candidate should exit 2, got $code"
grep -q 'cargo build-sbf' <<<"$doctor" || fail "doctor does not explain how to build the candidate"
grep -q 'needs no Rust' <<<"$doctor" || fail "doctor does not state Eplyx needs no Rust"
pass "init, runs and doctor guidance in a clean project"

# A valid checked-in package passes the offline doctor.
package="$root/examples/transitions/demo-fixed-ratio"
mkdir -p "$work/valid/target/deploy" && cd "$work/valid"
cp "$package/program.so" target/deploy/migration.so
field() { sed -n "s/.*\"$1\": *\"\([^\"]*\)\".*/\1/p" "$2" | head -n 1; }
source_mint=$(tr ',' '\n' < "$package/eplyx.json" | field sourceMint /dev/stdin)
replacement_mint=$(tr ',' '\n' < "$package/eplyx.json" | field replacementMint /dev/stdin)
owner=$(tr ',' '\n' < "$package/config.json" | field publicOwner /dev/stdin)
account=$(tr ',' '\n' < "$package/config.json" | field sourceAccount /dev/stdin)
cat > eplyx.toml <<TOML
[project]
name = "smoke"
[transition]
source_mint = "$source_mint"
replacement_mint = "$replacement_mint"
adapter = "fixed_ratio_conversion_v1"
effective_at = "2026-10-01T00:00:00Z"
[program]
path = "./target/deploy/migration.so"
[terms]
numerator = "1"
denominator = "2"
rounding = "floor"
fee_bps = 0
[execution]
public_owner = "$owner"
source_account = "$account"
amount_decimal = "0.000001"
reserve_funded_replacement_raw = "1000000000000"
[gate]
policy = "block-only"
TOML
"$bin" doctor --offline > "$work/doctor.txt" || { cat "$work/doctor.txt"; fail "doctor --offline rejected a valid package"; }
grep -q 'Transition package' "$work/doctor.txt" || fail "doctor did not validate the package"
pass "doctor --offline validates a real package"

# An existing Milestone 16 store opens unchanged: runs, show and the dashboard.
cp -R "$root/fixtures/dashboard/transition-acceptance" "$work/store"
mkdir -p "$work/store/.eplyx/cache" "$work/store/.eplyx/counterexamples" "$work/store/.eplyx/runs"
cd "$work/store"
runs=$("$bin" runs --json | grep -c '"run":' || true)
[ "$runs" = 4 ] || fail "expected 4 runs in the saved store, got $runs"
"$bin" show run_20260924123736483_ce7b4d55310c | grep -q 'Gate Block' || fail "show did not read the blocked run"
SOLANA_RPC_URL=https://provider.example/SMOKE-RPC-SECRET "$bin" dashboard --no-open --port 0 > "$work/dashboard.log" 2>&1 &
dashboard_pid=$!
url=""
for _ in $(seq 1 100); do
  url=$(grep -o 'http://127\.0\.0\.1:[0-9]*' "$work/dashboard.log" | head -n 1 || true)
  [ -n "$url" ] && break
  sleep 0.2
done
[ -n "$url" ] || { cat "$work/dashboard.log"; fail "dashboard did not start"; }
curl -fsS "$url/api/runs" > "$work/runs.json" || fail "dashboard API did not respond"
[ "$(grep -o '"id":"run_' "$work/runs.json" | wc -l | tr -d ' ')" = 4 ] || fail "dashboard did not list 4 runs"
curl -fsS "$url/" | grep -q '/assets/dashboard.js' || fail "dashboard page not served from the binary"
curl -fsS "$url/assets/dashboard.css" > /dev/null || fail "embedded dashboard assets missing"
[ "$(curl -s -o /dev/null -w '%{http_code}' -H 'Host: attacker.example' "$url/api/project")" = 403 ] || fail "foreign Host accepted"
if grep -rq 'SMOKE-RPC-SECRET' "$work/runs.json" "$work/dashboard.log"; then fail "RPC secret leaked"; fi
curl -fsS "$url/api/project" | grep -q 'SMOKE-RPC-SECRET' && fail "RPC secret leaked"
pass "saved store, show and dashboard on $url"

[ "$(sha "$archive")" = "$archive_sha" ] || fail "archive changed during smoke test"
pass "archive $(basename "$archive") sha256 $archive_sha unchanged"
