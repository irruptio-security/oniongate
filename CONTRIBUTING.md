# Contributing

OnionGate is GPL-3.0 software. By contributing, you agree that your contribution
is distributed under GPL-3.0.

## Development

```bash
npm ci
npm run deps
npm run check
cd src-tauri && cargo test && cargo clippy --all-targets -- -D warnings
```

Do not add downloaded executables, onion keys, bridge lines, logs, or local
security reports to commits.

## Pull requests

- Keep network mutations behind a typed platform API.
- Add rollback and startup-recovery behavior for every privileged mutation.
- Add tests for torrc, control-protocol, routing, firewall, and persistence
  changes.
- Describe privacy/security impact and failure behavior.
- Preserve attribution and verify license compatibility for reused code.
- Never weaken Tor circuits or silently fall back to a direct route.

Security vulnerabilities must follow [SECURITY.md](SECURITY.md), not public
issues.
