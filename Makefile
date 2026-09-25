SHELL := /bin/bash
CARGO := cargo

.PHONY: all build compare report fixtures test test-engine test-programs test-cloud fmt fmt-check lint

all: compare

build:
	./scripts/build-programs.sh

compare: build
	$(CARGO) run --locked -q -p eplyx-lifecycle-impact -- compare

report: build
	$(CARGO) run --locked -q -p eplyx-lifecycle-impact -- compare --format json --out report.json

fixtures:
	$(CARGO) run --locked -q -p eplyx-lifecycle-impact -- generate

test: test-programs test-engine

test-engine: build fixtures
	$(CARGO) test --locked --workspace

test-programs:
	./scripts/test-programs.sh

# Optional cloud workspace tests need a Postgres server; they never skip silently.
test-cloud:
	@test -n "$$EPLYX_CLOUD_TEST_DATABASE_URL" || { echo "error: set EPLYX_CLOUD_TEST_DATABASE_URL to a Postgres URL whose user may create databases, e.g. postgres://postgres@127.0.0.1:5432/postgres" >&2; exit 2; }
	$(CARGO) test --locked -p eplyx-cloud -- --include-ignored

fmt:
	$(CARGO) fmt --all
	$(CARGO) fmt --all --manifest-path programs/fixture-lending/Cargo.toml
	$(CARGO) fmt --all --manifest-path programs/eplyx-demo-conversion/Cargo.toml

fmt-check:
	$(CARGO) fmt --all -- --check
	$(CARGO) fmt --all --manifest-path programs/fixture-lending/Cargo.toml -- --check
	$(CARGO) fmt --all --manifest-path programs/eplyx-demo-conversion/Cargo.toml -- --check

lint:
	$(CARGO) clippy --locked --all-targets -- -D warnings
	$(CARGO) clippy --manifest-path programs/fixture-lending/Cargo.toml --no-default-features --features v1 --all-targets -- -D warnings
	$(CARGO) clippy --manifest-path programs/fixture-lending/Cargo.toml --no-default-features --features v2 --all-targets -- -D warnings
	$(CARGO) clippy --manifest-path programs/eplyx-demo-conversion/Cargo.toml --no-default-features --features no-entrypoint --all-targets -- -D warnings
