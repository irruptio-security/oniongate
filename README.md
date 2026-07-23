# OnionGate

**Route apps through isolated Tor circuits, expose localhost as a private onion
service, and verify that the workstation is not leaking.**

OnionGate is a GPL-3.0 Tor Workstation Toolkit for macOS, Linux, and an
experimental signed Windows adapter. It combines managed Tor, per-app routing,
an ephemeral Onion Lab, crash recovery, leak verification, and focused host
posture in one Tauri desktop application.

> OnionGate is not a VPN, Tor Browser, Tails, Whonix, an antivirus, or a general
> application firewall. It does not prevent browser fingerprinting or global
> traffic correlation. Read [THREAT_MODEL.md](THREAT_MODEL.md) before relying on
> it for sensitive work.

## Three things to try

### 1. Give two apps separate circuits

Open **Apps**, choose stable application bundles or executables, select **Only
selected via Tor**, enable TUN, and connect. OnionGate gives each app distinct
SOCKS authentication on Tor's `IsolateSOCKSAuth` listener. Rotate one app's
credentials without changing its stable bundle/path/signing identity.

### 2. Turn localhost into a private onion

Start a server bound to `127.0.0.1:3000`, open **Onion Lab**, and create a
private ephemeral service. OnionGate rejects wildcard listeners, discards the
service key on stop, generates v3 client authorization, and provides a QR/client
credential. Use **Test & audit** to check publication, latency, HTTP status, and
security headers.

### 3. Produce a redacted verification report

Open **Verify** and run the leak verifier. It checks Tor/direct egress
separation, Tor DNS, IPv6, UDP/QUIC containment, selected-app policy, Session
Guard, and crash-recovery state. Public IP values are compared in memory and are
not stored in SQLite or exported.

## Product pillars

- **Route:** managed Tor, trustworthy Smart Connect fallback, bridges,
  TUN/proxy modes, stable app identities, isolated circuits, and Session Guard.
- **Onion Lab:** secure loopback-to-v3-onion projects for local development,
  client authorization, QR handoff, audits, and explicit destruction.
- **Verify:** leak reports, live firewall inspection, emergency restoration,
  macOS posture, persistence baselines, and artifact inspection.

Optional **Harden** actions remain secondary, reversible, and clearly separated
from the core protection boundary.

## Why it is different

- **Tor Browser** remains the right choice for browser fingerprinting defenses.
  OnionGate focuses on non-browser applications and workstation diagnostics.
- **OnionHop** provides convenient Tor routing. OnionGate adds stable signed app
  identities, per-app `IsolateSOCKSAuth`, fail-closed Session Guard, a persisted
  mutation journal, and verifiable reports.
- **OnionShare** is excellent for chat and file sharing. OnionGate's Onion Lab
  targets local web/service development and deployment checks instead.
- **Tails/Whonix** provide stronger operating-system isolation. OnionGate is a
  practical toolkit for an existing workstation, not a replacement.
- **LuLu, BlockBlock, OverSight, and KnockKnock** remain specialist macOS tools.
  OnionGate detects and links to official installations instead of silently
  cloning or bundling them.

## Security behavior

- SOCKS hostname resolution uses `socks5h`; TUN DNS uses Tor's local UDP
  `DNSPort`.
- Smart Connect tries direct Tor, user-supplied BridgeDB lines, then bundled
  Snowflake. It does not consume untrusted GitHub bridge feeds.
- Every proxy, TUN, firewall, Tor, and transport mutation is journaled for
  startup recovery.
- Firewall status comes from live pf/nftables/Windows Firewall inspection, not
  only marker files.
- Onion keys and client credentials are never written to logs or SQLite.
- Sidecar archives are pinned in `scripts/dependencies.sha256`.

See [SECURITY.md](SECURITY.md), [THREAT_MODEL.md](THREAT_MODEL.md),
[ARCHITECTURE.md](ARCHITECTURE.md), [PRIVACY.md](PRIVACY.md), and
[RELEASE.md](RELEASE.md). Maintainers can use the
[demo script](docs/DEMO.md) and [comparison guide](docs/COMPARISON.md).

## Install

Release builds bundle Tor, lyrebird, and sing-box.

1. Download a signed release and its `SHA256SUMS`.
2. Verify the checksum and platform signature/notarization.
3. Open OnionGate and choose a preset: Everyday, Censored Network, Public
   Wi-Fi, Maximum Isolation, or Developer Lab.
4. Run **Verify** after connecting.

No stable release should be published until the release checklist and
fresh-machine recovery tests pass.

## Develop

Requirements: Node.js 24.18 Active LTS (see `.nvmrc`), Rust stable, Tauri
platform prerequisites, `curl`, `tar`, and `unzip` on Windows.

```bash
npm ci
npm run deps
npm run check
npm run tauri dev
```

Build:

```bash
npm run deps
npm run tauri build
```

CLI:

```bash
cd src-tauri
cargo run --bin tor-socks-cli -- status
cargo run --bin tor-socks-cli -- start
cargo run --bin tor-socks-cli -- stop
```

Linux aarch64 does not have an official Tor expert bundle; use
`ALLOW_MISSING_TOR=1` only for sing-box-only development. Windows builds require
signed release credentials and use WinTUN through sing-box.

## Contributing and releases

Contributions are licensed under GPL-3.0. Run TypeScript, Rust format, tests,
Clippy, and dependency audits before opening a pull request. Release CI produces
signed/notarized artifacts, updater signatures, checksums, and CycloneDX SBOMs;
see [CONTRIBUTING.md](CONTRIBUTING.md).

Bundled component notices and exact corresponding-source links are in
[`src-tauri/resources/licenses/THIRD_PARTY.md`](src-tauri/resources/licenses/THIRD_PARTY.md).
The local `priuvacy-sexy.txt` research artifact is not bundled.

## Trademark and affiliation

OnionGate is an independent project. It is not sponsored by, endorsed by, or
affiliated with The Tor Project. “Tor” and the onion logo are trademarks of The
Tor Project; OnionGate does not use the official Tor onion logo.
