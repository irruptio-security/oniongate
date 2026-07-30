---
layout: home
title: OnionGate
titleTemplate: false

hero:
  name: OnionGate
  text: A Tor workstation toolkit
  tagline: Route selected apps through Tor, host your own onion site, and inspect the live protection boundary.
  image:
    src: /logo.png
    alt: OnionGate
  actions:
    - theme: brand
      text: Quick start
      link: /guide/quick-start
    - theme: alt
      text: Host an onion site
      link: /guide/hosting
    - theme: alt
      text: View on GitHub
      link: https://github.com/irruptio-security/oniongate

features:
  - title: Per-app Tor circuits
    details: Selected TUN-routed apps get isolated Tor contexts. On macOS and Linux, Session Guard can suspend matching processes if that route disappears.
  - title: Onion Host
    details: Publish a loopback server as a v3 onion site. Temporary sites vanish for good when they stop; permanent sites keep the same address across restarts.
  - title: Leak verification
    details: Inspect egress plus live DNS, IPv6, UDP/QUIC, app-policy, and recovery state, with explicit warnings where a condition cannot be proven.
  - title: Crash recovery
    details: Protected-session proxy, TUN, firewall, Tor, and transport state is journaled and reversed during disconnect or Emergency Restore.
---

## Not a replacement for Tor Browser

OnionGate is **not** a VPN, Tor Browser, Tails, or Whonix. It does not stop
browser fingerprinting or global traffic correlation. Read the
[threat model](/reference/threat-model) before relying on it for sensitive work.

macOS is the primary workstation target. Linux integration is partial and
Windows is experimental. See [platform support](/reference/platform-support).

OnionGate has no stable release yet. Source builds and prereleases are alpha
software and must not be the sole control for high-risk activity.
