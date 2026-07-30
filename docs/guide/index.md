# What OnionGate is

OnionGate is a desktop app that routes individual applications through isolated
Tor circuits, turns a local port into an onion site, and checks whether its live
routing controls and egress are behaving as expected. It bundles and manages
Tor for you, so no terminal work is required.

macOS is the primary workstation target, Linux has partial desktop integration,
and Windows is experimental. Read the
[platform matrix](/reference/platform-support) before choosing a protection
boundary. A headless [command-line companion](/guide/cli) supports connection
to managed Tor and permanent onion hosting, with documented pre-stable limits.

## What you can do with it

**Route apps through Tor.** Give each application its own isolated circuit. A
fail-closed Session Guard suspends protected apps instead of leaking if Tor
drops.

See [Connect and route traffic](/guide/connection) and
[Route applications](/guide/apps) for the boundary and platform limitations.

**Host an onion site.** Expose `127.0.0.1:<port>` as a v3 onion service with
client authorization and a QR handoff. Choose a temporary site that disappears
for good when it stops, or a permanent one that keeps the same address across
restarts. See [Host an onion site](/guide/hosting).

**Inspect the live boundary.** Run diagnostics covering DNS, IPv6, UDP/QUIC,
and per-app policy, then export a redacted report. Verification is a live
configuration and egress diagnostic, not a packet capture or formal proof.

**Check and harden the machine itself.** Read your macOS security state, apply
reversible privacy and security changes, and watch what runs at startup. See
[Check and harden this machine](/guide/system).

## What it is not

OnionGate is not a VPN, Tor Browser, Tails, Whonix, an antivirus, or a general
application firewall. Specifically, it does **not**:

- protect against browser fingerprinting — use Tor Browser for that;
- defend against a global adversary correlating traffic;
- make an unsafe application protocol anonymous;
- contain malware or replace endpoint security;
- tunnel UDP, which is blocked rather than leaked.
- make Windows a stable fail-closed platform;
- prove that every process or packet followed the intended route.

Read the [threat model](/reference/threat-model) before relying on it for
anything sensitive.

## Next steps

- [Install OnionGate](/guide/install)
- [Quick start](/guide/quick-start)
- [Connect and route traffic](/guide/connection)
- [Use bridges](/guide/bridges)
- [Route applications](/guide/apps)
- [Host an onion site](/guide/hosting)
- [Verify the live boundary](/guide/verify)
- [Settings and logs](/guide/settings)
- [Recovery and troubleshooting](/guide/troubleshooting)
- [Use the command line](/guide/cli)
