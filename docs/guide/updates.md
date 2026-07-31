# Local installers and updates

## Build download files locally

Run this on the operating system you want to package:

```bash
make setup       # first time
make downloads
```

The build is unsigned and intended for local testing. The command prints the
exact output paths:

- macOS: `src-tauri/target/<target>/release/bundle/dmg/*.dmg`
- Windows: `src-tauri/target/<target>/release/bundle/nsis/*-setup.exe`
- Linux: AppImage, DEB, and RPM directories under
  `src-tauri/target/<target>/release/bundle/`

On macOS the local target uses non-interactive DMG layout mode. It does not need
Cursor/Terminal automation permission to control Finder.

The DMG contains `OnionGate.app`. A macOS machine builds its own architecture;
Windows and Linux installers should be built natively on those systems. GitHub
release CI builds every supported target with signing and provenance.

::: warning Local bundles are unsigned
`make downloads` deliberately passes `--no-sign`. Gatekeeper, Authenticode, and
the Tauri updater trust chain apply only to official CI artifacts.
:::

## When to create an update

The first published version is an installation, not an update. Create an update
when users already have an older signed OnionGate release and you are ready to
publish a newer version.

Before tagging:

1. invoke the project `release-changelog` Cursor skill with the next semantic
   version;
2. review and merge its version/changelog change on `main` or `staging`;
3. run `make release-check VERSION=<version>`;
4. tag that exact commit as `v<version>` and push the tag.

Release CI creates a draft. Test the installers, checksums, SBOMs, and
attestations before publishing.

## How automatic updates work

The installed app contains only the updater **public** key. The matching private
key stays offline and in GitHub Actions secrets.

When the user selects **Settings → Check for updates**:

1. OnionGate requests
   `https://github.com/irruptio-security/oniongate/releases/latest/download/latest.json`;
2. the manifest selects the current OS and architecture;
3. Tauri downloads the updater payload;
4. the embedded public key verifies its signature;
5. OnionGate installs it and relaunches.

A missing or invalid signature fails the update. OnionGate never bypasses the
check.

## Stable and staging channels

Tags must point to commits on `main` or `staging`.

- `main` produces the normal release channel.
- `staging` always produces a prerelease for manual testing.

GitHub's `/releases/latest/` endpoint excludes drafts and prereleases. Therefore
the built-in updater follows published non-prerelease releases only. Staging
installers are downloaded and tested manually; they do not replace the stable
update channel.

## Trust-root warning

Losing `TAURI_SIGNING_PRIVATE_KEY` or its password prevents existing
installations from accepting future updates. Rotating the public key in
`tauri.conf.json` also breaks that trust chain. Keep an encrypted offline backup
and never commit either secret.
