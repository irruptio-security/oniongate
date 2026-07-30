# OnionGate — common developer entry points
# Run `make help` for the full list.

.PHONY: help setup install deps \
	start dev run build build-frontend preview \
	check typecheck test lint fmt fmt-check audit \
	clean clean-rust clean-deps

CARGO_MANIFEST := src-tauri/Cargo.toml
NPM ?= npm

help: ## Show this help
	@printf 'OnionGate targets\n\n'
	@awk 'BEGIN {FS = ":.*## "}; /^[a-zA-Z0-9_-]+:.*## / {printf "  %-18s %s\n", $$1, $$2}' $(MAKEFILE_LIST)

# ---------------------------------------------------------------------------
# Install from source
# ---------------------------------------------------------------------------

setup: ## Install npm deps and fetch verified Tor / sing-box sidecars
	$(NPM) ci
	$(NPM) run deps

install: setup ## Alias for setup (install from source)

deps: ## Download + SHA-256-verify bundled Tor / lyrebird / sing-box
	$(NPM) run deps

# ---------------------------------------------------------------------------
# Develop / run
# ---------------------------------------------------------------------------

start: deps ## Start the Tauri app (development mode)
	$(NPM) run tauri dev

dev: start ## Alias for start

run: start ## Alias for start

preview: ## Serve the built frontend (vite preview)
	$(NPM) run preview

# ---------------------------------------------------------------------------
# Build
# ---------------------------------------------------------------------------

build: deps ## Build a release app bundle (Tauri)
	$(NPM) run tauri build

build-frontend: ## Typecheck and build the Vite frontend only
	$(NPM) run build

# ---------------------------------------------------------------------------
# Quality
# ---------------------------------------------------------------------------

check: ## Typecheck frontend + run Rust tests
	$(NPM) run check

typecheck: ## TypeScript check (no emit)
	$(NPM) run typecheck

test: ## Run Rust tests
	cargo test --manifest-path $(CARGO_MANIFEST)

lint: ## Clippy (all targets) + rustfmt check
	$(NPM) run lint:rust
	$(NPM) run fmt:check

fmt: ## Format Rust sources
	cargo fmt --manifest-path $(CARGO_MANIFEST) --all

fmt-check: ## Check Rust formatting without writing
	$(NPM) run fmt:check

audit: ## cargo audit against src-tauri/Cargo.lock
	cargo audit --file src-tauri/Cargo.lock

# ---------------------------------------------------------------------------
# Clean
# ---------------------------------------------------------------------------

clean: clean-rust ## Remove Rust build artifacts
	@echo "Done. (node_modules and .deps-cache kept; use clean-deps to drop sidecars)"

clean-rust: ## cargo clean for src-tauri
	$(NPM) run clean:rust

clean-deps: ## Remove downloaded sidecars and the deps cache
	rm -rf .deps-cache
	find src-tauri/binaries -mindepth 1 -maxdepth 1 ! -name 'README.md' -exec rm -rf {} +
	rm -rf src-tauri/resources/runtime
