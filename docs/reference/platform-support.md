# Platform support

OnionGate is pre-stable software. “Implemented” below means a platform backend
exists and is exercised by CI; it does not mean the project has completed an
independent security audit or published a stable signed release.

| Capability | macOS | Linux | Windows |
| --- | --- | --- | --- |
| Managed Tor and control port | Implemented | Implemented | Experimental |
| Temporary/permanent Onion Host | Implemented | Implemented | Experimental |
| System SOCKS proxy | Implemented | GNOME only | WinINet-aware apps only |
| sing-box TUN | Implemented | Implemented | Experimental |
| UDP/QUIC kill switch | `pf` | `nftables` | Defender Firewall |
| Selected-app TUN rules | Implemented | Implemented | Experimental |
| Session Guard process suspension | Implemented | Implemented | Not supported |
| App detection and bypass helpers | Implemented | Implemented | Not supported |
| Workstation Checkup / Startup Items | Implemented | Not supported | Not supported |
| OS hardening | Implemented | Not supported | Not supported |
| Privileged-helper service code | Implemented | Implemented | Experimental |
| CI compile/test | `macos-14` | Ubuntu latest | Windows latest |

## macOS

macOS is the primary workstation target.

- System proxy changes use `networksetup` and capture up to three active network
  services for restoration.
- TUN uses elevated sing-box and may appear as a `utun` interface even though
  the generated interface name is `torsocks0`.
- The kill switch uses a dedicated `pf` anchor and blocks outbound UDP except
  loopback.
- Session Guard uses process stop/continue signals.
- Checkup, persistence baselines, Background/Login Items scanning, and all
  current hardening controls are macOS-specific.
- Background/Login Items scanning is explicit and requires Full Disk Access; it
  is never run automatically.

Normal downloadable builds also need Developer ID signing, hardened runtime,
notarization, and stapling. Until those artifacts are published, source builds
should be treated as development software.

## Linux

Linux support depends on the desktop and host tools:

- System proxy integration supports GNOME `gsettings`; other desktop
  environments report it unavailable.
- TUN needs the host's TUN support and an elevation path.
- The kill switch requires `nft`. Without the helper, elevation uses `pkexec` or
  non-interactive/passwordless `sudo`.
- App detection searches common binaries and `.desktop` locations.
- Session Guard needs `pgrep` and `kill`.
- Workstation Checkup, persistence baselines, login-item scanning, and OS
  hardening are not implemented.

The CI target is Ubuntu, not every distribution. Package names, PolicyKit,
systemd, firewall tooling, and WebKit availability vary by distribution.

The Tor Project's 15.0.x expert bundle does not publish a Linux AArch64 archive.
`make deps` therefore fails closed on `aarch64-unknown-linux-gnu` unless a
developer explicitly sets `ALLOW_MISSING_TOR=1`; that override produces no
usable bundled Tor and is not a release path.

## Windows

Windows remains experimental:

- The system proxy changes WinINet registry values. Software that uses WinHTTP,
  custom DNS, or its own network stack may ignore them.
- TUN and selected-app policy use sing-box, but Session Guard process suspension
  is not implemented.
- The kill switch creates a named outbound UDP rule in Windows Defender
  Firewall.
- Workstation Checkup, app bypass helpers, and OS hardening are not implemented.
- Release artifacts may be unsigned until an Authenticode certificate is
  configured and must never be labeled stable.

Do not rely on the Windows build for a high-risk fail-closed workflow.

## Privileged helper packaging status

The repository contains install/service code for `oniongate-helper`, and its
current protocol is limited to typed kill-switch operations. The helper must be
built, placed beside the app, and signed with the same identity as the app.

Release CI now builds the helper per target and applies a release-only Tauri
sidecar overlay so development builds do not depend on a staged helper. The
draft must still be inspected to confirm the helper is present and shares the
application's platform signature. If absent, installation fails explicitly and
OnionGate uses interactive elevation instead.

Before stable release, the helper must also move to a minimal crate, verify the
macOS client code signature/audit token rather than UID alone, and enforce a
reviewed Windows pipe ACL and client identity.

## Support policy

Only the latest tagged release is eligible for security fixes. There is
currently no stable tagged release. All source and prerelease builds are alpha
software and must not be used as the sole control for high-risk activity.

See the [security policy](https://github.com/irruptio-security/oniongate/blob/main/SECURITY.md)
and [threat model](/reference/threat-model).
