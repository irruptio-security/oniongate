# Release process

OnionGate has not published a stable release. The workflow intentionally creates
a **draft** first so artifacts can be inspected before publication. Version
`0.x` and hyphenated versions are also marked prerelease.

## Cursor changelog gate

Before creating a tag, invoke the project `release-changelog` Cursor skill with
the intended semantic version. Cursor reviews every commit and full diff since
the previous tag, updates `CHANGELOG.md`, synchronizes package/Cargo/Tauri
versions, and runs the release preflight.

Review and merge that change before tagging. Release CI rejects:

- a tag whose commit is not reachable from `main` or `staging`;
- a tag that differs from the application version;
- mismatched versions across package, lockfile, Cargo, or Tauri config;
- a missing dated `CHANGELOG.md` section for that version.

Cursor prepares text; a maintainer remains responsible for accuracy. AI output
must never contain secrets, bridge lines, onion/client credentials, public IPs,
local paths, or private report content.

`main` and `staging` are the only branches allowed to run normal CI or produce
downloadable artifacts. A tag reachable only from `staging` always creates a
prerelease. If the commit is reachable from both branches, it is treated as a
`main` release.

## Current automation

Pushing a tag matching `v*` first runs typecheck, format, Clippy, Rust tests,
docs build, npm production audit, Cargo audit, and the version/changelog gate.
Only then does it create or refresh a draft release and build:

- Apple Silicon on `macos-15` (`aarch64-apple-darwin`);
- Intel on `macos-15-intel` (`x86_64-apple-darwin`);
- Ubuntu 22.04;
- Windows Server 2022.

The manual-install assets are two architecture-specific macOS DMGs containing
`OnionGate.app`, a Windows NSIS setup EXE, and Linux AppImage/DEB/RPM packages.
Tauri also uploads signed platform updater payloads and `latest.json`; those
support in-app updates and are not substitutes for the primary installers.

Each target downloads verified sidecars, builds and stages
`oniongate-helper`, creates updater signatures, and uploads installers to the
same draft. Builds are serialized because each target merges its entry into
`latest.json`; parallel writes can silently drop platforms.

The metadata job requires all four updater platform keys, then:

- generates Cargo, npm, and pinned-sidecar CycloneDX SBOMs;
- creates `SHA256SUMS` and signs it with the updater trust root;
- publishes GitHub build-provenance attestations;
- uploads all evidence as release assets and a retained workflow artifact.

macOS credentials and the Tauri updater key are mandatory. Windows may build
without Authenticode only while Windows remains explicitly experimental.

Publishing a non-prerelease triggers a separate post-publish check of the public
`releases/latest/download/latest.json` endpoint, release checksums, and GitHub
attestations. A failed post-publish check is a release incident.

## Known release blockers

The workflow still does **not**:

- run quarantined-install smoke tests on fresh VMs;
- publish the draft or mark it Latest without human artifact review;
- give CLI `start` the same verified TUN/firewall/proxy orchestration and
  long-running session ownership as the desktop app;
- provide manageable cross-process lifecycle for CLI-created temporary sites.
- complete the helper's minimal-crate and client code-signature hardening.

Consequently, artifacts remain draft/prerelease until the manual gates below are
completed and must not be described as stable or production-ready.

## Updater trust root

The Tauri updater public key is embedded in `src-tauri/tauri.conf.json`. Its
private key and password must exist only in offline backup and GitHub Actions
secrets:

- `TAURI_SIGNING_PRIVATE_KEY`
- `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`

Losing the private key prevents installed clients from trusting future updates.
Replacing the public key casually breaks the update chain. Never commit either
the private key or password.

## Platform credentials

macOS requires:

- `APPLE_CERTIFICATE`
- `APPLE_CERTIFICATE_PASSWORD`
- `APPLE_SIGNING_IDENTITY`
- `APPLE_TEAM_ID`
- `APPLE_API_KEY`
- `APPLE_API_ISSUER`
- `APPLE_API_KEY_BASE64`

Windows signing uses:

- `WINDOWS_CERTIFICATE`
- `WINDOWS_CERTIFICATE_PASSWORD`

The full setup procedure lives in
[`.github/RELEASE_SECRETS.md`](https://github.com/irruptio-security/oniongate/blob/main/.github/RELEASE_SECRETS.md).
That file documents names and procedures only; values stay in repository
secrets.

## Sidecar update procedure

1. Update one dependency/version at a time.
2. Download the immutable upstream archive from its official source.
3. For Tor, verify the Tor Project's signed checksum manifest.
4. Independently reproduce or verify the archive hash.
5. Update `scripts/dependencies.sha256`.
6. Run `make clean-deps && make deps` on every affected architecture.
7. Confirm staged filenames match Tauri's target-triple `externalBin` names.
8. Update third-party notices and corresponding-source links.
9. Review the sidecar's release notes for protocol, configuration, and license
   changes.

Never make the dependency script accept an unpinned archive.

## Stable-release gate

Before publishing a stable tag:

1. Run `make check`, `make lint`, `make audit`, `make build-frontend`, and
   `make docs-build` from a clean checkout.
2. Ensure CI passes on macOS, Linux, and Windows.
3. Review the threat model, privacy inventory, platform matrix, and installer
   docs in the same commit.
4. Build every artifact from the tag in GitHub Actions.
5. Package and sign the helper and all nested executable sidecars.
6. Sign/notarize/staple macOS; Authenticode-sign Windows.
7. Generate SHA-256 checksums, detached signatures, SBOMs, licenses, and build
   provenance.
8. Verify every uploaded artifact against the checksums.
9. Test a quarantined download and uninstall/recovery on clean macOS, Linux, and
   Windows VMs.
10. Exercise Connect, TUN, kill switch, Emergency Restore, temporary/permanent
    Onion Host, client authorization, and updater behavior.
11. Verify `latest.json` contains macOS ARM64/Intel, Linux x86_64, and Windows
    x86_64 entries with valid signatures.
12. Publish the draft only after all gates pass; mark Latest only for a stable
    release.

GitHub's `macos-15-intel` runner is scheduled to retire in August 2027. Intel
macOS releases need a replacement build runner or an explicit end-of-support
decision before then.

## Documentation deployment

Documentation is independent of application releases. A push to `main` that
changes `docs/`, package manifests, or the docs workflow builds VitePress and
deploys it to GitHub Pages. Pull requests build the site without deploying it.
