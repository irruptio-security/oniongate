<p align="center">
  <img src="public/logo.png" alt="OnionGate" width="120" height="120" />
</p>

<h1 align="center">OnionGate</h1>

<p align="center">
  <b>A Tor workstation toolkit for macOS, Linux, and Windows (experimental).</b>
</p>

OnionGate is a desktop app that routes individual applications through isolated
Tor circuits, turns a local port into a private onion service, and verifies that
your machine isn't leaking outside Tor. It bundles and manages Tor for you — no
terminal required.

> OnionGate is **not** a VPN, Tor Browser, Tails, or Whonix. It does not stop
> browser fingerprinting or global traffic correlation. Read the
> [threat model](THREAT_MODEL.md) before relying on it for sensitive work.

## What you can do

- **Route apps through Tor** — give each app its own isolated circuit, with a
  fail-closed Session Guard that suspends protected apps instead of leaking if
  Tor drops.
- **Host a private onion** — expose `127.0.0.1:<port>` as an ephemeral v3 onion
  service with client authorization and a QR handoff, then destroy it (and its
  key) in one click. See the [Onion Lab guide](docs/ONION_LAB.md).
- **Verify you're safe** — run leak checks (DNS, IPv6, UDP/QUIC, per-app policy)
  and export a redacted report. Your public IP is compared in memory, never
  stored.

## OnionGate vs OnionHop

OnionHop makes it easy to send your traffic through Tor. OnionGate goes further
for people who need stronger guarantees:

- **Per-app isolation** — a separate Tor circuit per application, not one shared
  route.
- **Fail-closed Session Guard** — protected apps stop rather than fall back to a
  direct connection.
- **Crash recovery** — every network change is journaled and restored after a
  crash or interrupted disconnect.
- **Onion Lab** — build and audit private onion services for local development.
- **Verifiable reports** — redacted leak reports you can inspect and export.

## Install

Release builds bundle Tor, lyrebird, and sing-box.

1. Download a signed release and its `SHA256SUMS`.
2. Verify the checksum and platform signature.
3. Open OnionGate, pick a preset, and run **Verify** after connecting.

## Develop

Requires Node.js (see `.nvmrc`), Rust stable, Make, and the Tauri prerequisites
for your OS. Prefer the Makefile targets (`make help` lists them all).

```bash
make setup          # npm ci + download/verify Tor / sing-box sidecars
make start          # tauri dev
```

Build a release bundle from source:

```bash
make build
```

Quality checks before a PR:

```bash
make check
make lint
```

## Contributing

Contributions are welcome under GPL-3.0. Please read
[CONTRIBUTING.md](CONTRIBUTING.md) and our
[Code of Conduct](CODE_OF_CONDUCT.md) first. Report security issues privately per
[SECURITY.md](SECURITY.md) — never in a public issue.

## Documentation

- [Onion Lab: host your own onion endpoint](docs/ONION_LAB.md)
- [Architecture](ARCHITECTURE.md)
- [Threat model](THREAT_MODEL.md)
- [Security policy](SECURITY.md)
- [Privacy](PRIVACY.md)
- [Release process](RELEASE.md)

## License & trademark

OnionGate is licensed under [GPL-3.0](LICENSE). Bundled component notices and
corresponding-source links are in
[`THIRD_PARTY.md`](src-tauri/resources/licenses/THIRD_PARTY.md).

OnionGate is an independent project — not affiliated with or endorsed by The Tor
Project. "Tor" and the onion logo are trademarks of The Tor Project; OnionGate
uses its own logo, not the official Tor onion logo.
