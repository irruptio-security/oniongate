# Install

## Current release status

OnionGate has not published a stable release. Treat source builds and any future
draft/prerelease artifacts as alpha software. Do not rely on them for high-risk
activity.

When signed artifacts become available, use only the
[official releases page](https://github.com/irruptio-security/oniongate/releases).
A production release is expected to bundle Tor, sing-box, lyrebird, obfs4proxy,
and the required runtime resources.

Primary downloads:

| Platform | Install asset |
| --- | --- |
| macOS Apple Silicon / Intel | `.dmg` containing `OnionGate.app`; open it and drag the app to Applications |
| Windows x86_64 | Authenticode-signed NSIS `-setup.exe` installer |
| Linux x86_64 | `.AppImage`, with `.deb` and `.rpm` packages for supported distributions |

Files such as `OnionGate.app.tar.gz`, `.sig`, and `latest.json` are signed
in-app-updater support assets, not the normal manual installer.

Before running a downloaded artifact:

::: warning No artifact has been published yet
Release CI now prepares signed draft installers, checksums, SBOMs, and
provenance, but a maintainer must test and publish the draft. These commands
apply only after assets appear on the official Releases page. Do not treat a
draft or prerelease as production-ready because one check happens to pass.
:::

```bash
# macOS
shasum -a 256 -c SHA256SUMS --ignore-missing

# Linux
sha256sum -c SHA256SUMS --ignore-missing
```

Also verify the platform signature:

- macOS:
  ```bash
  codesign --verify --deep --strict --verbose=2 /Applications/OnionGate.app
  spctl --assess --type execute --verbose=4 /Applications/OnionGate.app
  xcrun stapler validate /Applications/OnionGate.app
  ```
- Windows PowerShell:
  ```powershell
  Get-AuthenticodeSignature .\OnionGate.exe | Format-List
  ```
  Require a valid status and the publisher named in the release notes.
- Linux: published checksum/signature and matching release provenance.

If a checksum, signature, SBOM, or provenance promised by the release notes is
missing, stop rather than bypassing the check.

When GitHub CLI is available, verify the build attestation too:

```bash
gh attestation verify <downloaded-artifact> \
  --repo irruptio-security/oniongate
```

## From source

You need Node.js (see `.nvmrc`), Rust stable, Make, and the
[Tauri prerequisites](https://tauri.app/start/prerequisites/) for your platform.

```bash
git clone --depth 1 https://github.com/irruptio-security/oniongate.git
cd oniongate
make setup          # npm ci + download and verify the bundled Tor / sing-box runtimes
make start          # run the app in development mode
```

`make setup` downloads executable sidecars. Read the hash manifest and script
before running it if you are evaluating the supply chain.

To build a release bundle locally:

```bash
make build
```

Local bundles are not automatically signed, notarized, or trusted by the
updater. A successful local build is not equivalent to an official release.

Run `make help` for the full list of targets.

### Why `make setup` downloads binaries

OnionGate bundles known Tor and transport versions rather than trusting whatever
is on the host. `make setup` fetches archives and checks each one against a
pinned SHA-256 in `scripts/dependencies.sha256`. A mismatched or unpinned archive
is refused, so the build fails loudly instead of staging an unverified binary.

The fetched runtime directories are ignored by Git. Never commit them manually.

## Platform notes

### macOS

Install Apple's command-line build tools and the Tauri prerequisites. TUN,
firewall, proxy, helper installation, and many hardening controls may ask for
administrator approval.

Gatekeeper warnings are expected for an unsigned local build. Do not disable
Gatekeeper globally; use a signed official release when one exists.

### Linux

The Tauri build needs WebKitGTK and distribution build packages. The system
proxy backend currently supports GNOME `gsettings`. TUN needs host TUN support,
and the kill switch requires `nftables` plus an elevation path (`pkexec` or
appropriate sudo policy).

### Windows

Windows is experimental. The WinINet proxy does not cover every application,
and Session Guard process suspension is unavailable. Unsigned artifacts must
not be labeled stable or used for high-risk workflows.

See the complete [platform support matrix](/reference/platform-support).

## First launch

1. Read the [threat model](/reference/threat-model).
2. Complete the setup wizard or choose a preset under Settings.
3. Connect Tor and wait for 100% bootstrap.
4. Select Proxy or TUN based on the boundary you need.
5. Run **Verify** and read every warning.

OnionGate stores application state under a legacy internal
`tor-socks-gui` data-directory name. Do not place it in a synced folder; it can
contain permanent onion keys.

## The command line

The CLI is built alongside the app as the `oniongate` binary:

```bash
cargo build --release --manifest-path src-tauri/Cargo.toml --bin oniongate
```

See the [CLI guide](/guide/cli) for what it can do.
