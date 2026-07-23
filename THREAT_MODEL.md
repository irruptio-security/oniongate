# OnionGate threat model

## Goals

OnionGate aims to:

1. route selected TCP applications through Tor without DNS or UDP leaks;
2. fail closed for applications explicitly placed under Session Guard;
3. restore network state after disconnect, crash, or interrupted elevation;
4. create temporary v3 onion services for loopback development listeners;
5. make the active protection boundary observable and testable.

## Adversaries considered

- A local network or ISP observing destinations and blocking Tor.
- A remote service attempting to learn the client's public IP.
- A malicious or unreliable Tor exit observing unencrypted application data.
- An application accidentally bypassing proxy settings.
- An attacker who learns an onion address but lacks its client credential.
- A crash that leaves proxy, TUN, firewall, or helper state behind.
- A compromised dependency or release artifact.

## Out of scope

- A compromised operating-system kernel, root account, firmware, or hardware.
- Browser fingerprinting and state isolation equivalent to Tor Browser.
- Global passive traffic correlation.
- Making unsafe application protocols anonymous.
- Malware prevention, exploit containment, or a general endpoint-security
  replacement.
- UDP tunnelling, torrenting, streaming, gaming, or stable geolocation.
- Protecting a local onion service that is itself vulnerable.

## Modes

### SOCKS proxy

Only applications that use `socks5h` or otherwise delegate hostname resolution
to Tor are protected. The operating-system proxy is a compatibility feature,
not a complete containment boundary.

### TUN

The TUN captures system traffic, sends supported TCP through Tor SOCKS, and
blocks unsupported UDP/QUIC. Private-LAN bypasses, split policies, IPv6, DNS,
and route changes must be covered by automated leak tests.

### Session Guard

Guarded applications receive isolated SOCKS credentials and must not use a
direct fallback if Tor, TUN, or the route policy becomes unavailable.

### Onion Lab

Services bind only to loopback. Ephemeral services discard their key at stop.
Private services require v3 client authorization. Persistent keys are an
explicit advanced feature and must be stored with owner-only permissions.

## Sensitive data

Never include bridge lines, onion private keys, client credentials, local file
paths, destination history, or full process command lines in telemetry or
diagnostic exports. OnionGate has no analytics by default.

## Release assurances

Release artifacts must be built by CI from a tag, signed, checksummed, supplied
with an SBOM, and created from sidecars verified against the pinned dependency
manifest. A notarized build is not a substitute for a security review.
