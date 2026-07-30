<!-- Thanks for contributing to OnionGate. Contributions are licensed under GPL-3.0. -->

## Summary

<!-- What does this change do, and why? -->

## Privacy / security impact

<!-- Describe the effect on routing, leak behavior, and failure modes.
     State "none" only if truly none. -->

## Checklist

- [ ] `npm run check` passes (typecheck + Rust tests)
- [ ] `cargo fmt --check` and `cargo clippy --all-targets -- -D warnings` pass
- [ ] Network mutations stay behind a typed platform API with rollback + startup recovery
- [ ] Added tests for torrc / control-protocol / routing / firewall / persistence changes
- [ ] No secrets, onion keys, bridge lines, logs, or local reports committed
- [ ] Does not weaken Tor circuits or add a silent direct-route fallback
- [ ] Attribution preserved and license compatibility verified for reused code

<!-- Report security vulnerabilities privately per SECURITY.md, not in a public PR. -->
