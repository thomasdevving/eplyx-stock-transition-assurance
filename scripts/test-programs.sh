#!/usr/bin/env bash
# Host-target unit tests for the fixture program, run once per build flavour.
# These cover the arithmetic in isolation; the differential suite in
# engine/tests covers it through real SBF execution.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
MANIFEST="$ROOT/programs/fixture-lending/Cargo.toml"
[ -d "$HOME/.cargo/bin" ] && export PATH="$HOME/.cargo/bin:$PATH"

for VERSION in v1 v2; do
  echo "==> testing fixture-lending [$VERSION]"
  cargo test --manifest-path "$MANIFEST" --no-default-features --features "$VERSION"
done

# The registered candidate conversion mechanism: its exact ratio, rounding and fee
# arithmetic in isolation. Real SBF execution is covered by engine/tests.
echo "==> testing eplyx-demo-conversion [candidate]"
cargo test --manifest-path "$ROOT/programs/eplyx-demo-conversion/Cargo.toml" \
  --no-default-features --features no-entrypoint
