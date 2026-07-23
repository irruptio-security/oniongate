# Architecture

OnionGate is a Tauri 2 application with a React UI and Rust core.

## Trust tiers

1. **UI:** renders status and sends typed Tauri commands.
2. **Core:** owns settings, Tor control, policy, diagnostics, and session state.
3. **Platform adapters:** proxy, TUN, firewall, elevation, and host probes.
4. **Sidecars:** Tor, lyrebird, and sing-box pinned in
   `scripts/dependencies.sha256`.
5. **Future helper:** a signed, narrow privileged service for crash-safe policy;
   it must not accept arbitrary shell.

## Core modules

- `tor/`: managed Tor, control protocol, bridges, and transports.
- `tun/`: sing-box TUN and per-application route generation.
- `proxy/`: reversible operating-system SOCKS configuration.
- `firewall/`: leak guard and live platform state.
- `session/`: persisted mutation journal and recovery.
- `onion_service/`: ephemeral v3 onion service lifecycle.
- `verify/`: local leak and workstation posture diagnostics.
- `harden/`: explicit, reversible host settings outside the core protection
  boundary.

The GUI and CLI must call the same orchestration APIs. No command may claim a
protected state until all required components have been verified live.
