# Honest product comparison

## Tor Browser

Use Tor Browser for web browsing when fingerprinting resistance and browser
state isolation matter. OnionGate does not reproduce those defenses. OnionGate
is useful for non-browser TCP applications, local onion development, route
recovery, and workstation verification.

## OnionHop

Both products make desktop Tor routing convenient. OnionGate's technical focus
is stable application identity, per-app `IsolateSOCKSAuth`, Session Guard,
persisted rollback, Onion Lab, and redacted verification reports.

## OnionShare

Use OnionShare for mature file sharing, chat, and receive workflows. OnionGate
does not clone them. Onion Lab exposes a developer's existing loopback service
and audits it before deployment.

## Tails and Whonix

Use Tails or Whonix when a dedicated operating-system security boundary is
appropriate. OnionGate runs on the user's existing workstation and cannot
provide equivalent isolation.

## LuLu, BlockBlock, OverSight, and KnockKnock

These Objective-See tools are specialist macOS controls. OnionGate detects and
links to their official releases. It does not bundle them or claim that unknown
persistence is malware.

## Portmaster

OnionGate is not a general application firewall or DNS policy engine. Its
Session Guard is deliberately Tor-aware and narrow. A future Network Extension,
nftables/eBPF, or WFP firewall requires separate entitlement, helper, policy,
and security-review work.
