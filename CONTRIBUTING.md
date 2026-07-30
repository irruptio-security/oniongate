# Contributing

OnionGate is GPL-3.0 software. By contributing, you agree that your contribution
is distributed under GPL-3.0. Participation is governed by our
[Code of Conduct](CODE_OF_CONDUCT.md).

## Development

```bash
make setup
make check
make lint
```

(`make help` lists setup, dev, build, clean, and other targets.)

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
