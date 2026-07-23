# Bundled binaries

Populated by `scripts/download-deps.sh`.

Tauri `externalBin` expects names like:

- `tor-aarch64-apple-darwin`
- `sing-box-aarch64-apple-darwin`
- `lyrebird-aarch64-apple-darwin`
- `obfs4proxy-aarch64-apple-darwin` (alias of lyrebird)
- `conjure-client-aarch64-apple-darwin` (when present)

Do not commit large binaries; CI/release builders run the download script.
