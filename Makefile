SHELL := /bin/bash

.PHONY: all preflight build release test clean validate
all: build

# preflight: ensure code is formatted and lint-free before other actions
preflight:
	@echo "running cargo fmt (check)"
	@cargo fmt -- --check
	@echo "running cargo clippy (deny warnings)"
	@cargo clippy -- -D warnings || (echo "cargo clippy failed; install clippy or fix warnings"; exit 1)

build: preflight
	@echo "cargo build --workspace"
	@cargo build --workspace


release:
	@echo "cargo build --release --workspace"
	@cargo build --release --workspace

test: preflight
	@echo "cargo test --workspace"
	@cargo test --workspace

clean:
	@echo "cargo clean"
	@cargo clean
	@echo "remove generated samples"
	@rm -f sample.fodt sample.odt

# validate: generate sample documents and run Relax-NG validation.
# It prefers xmllint (libxml2). If xmllint is not available it will try jing.
# If ODFERROUS_RNG_STRICT=1 and no validator is found the target fails.

validate: preflight


validate:
	@echo "Generating sample documents..."
	@cargo run --bin generate >/dev/null
	@echo "Validating sample.fodt against OpenDocument-v1.4-schema.rng"
	@if command -v xmllint >/dev/null 2>&1; then \
		xmllint --noout --relaxng OpenDocument-v1.4-schema.rng sample.fodt && echo "xmllint: sample.fodt validates"; \
	elif command -v jing >/dev/null 2>&1; then \
		echo "xmllint not found, using jing"; \
		jing OpenDocument-v1.4-schema.rng sample.fodt && echo "jing: sample.fodt validates"; \
	else \
		if [ "$$ODFERROUS_RNG_STRICT" = "1" ]; then \
			echo "No XML validator (xmllint or jing) found and ODFERROUS_RNG_STRICT=1; failing"; exit 1; \
		else \
			echo "No XML validator found (xmllint or jing). To make this fatal set ODFERROUS_RNG_STRICT=1"; \
		fi; \
	fi
