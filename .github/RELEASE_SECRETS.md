# Release credentials

Never paste credential values into issues, commits, pull requests, or chat.
Configure them as GitHub Actions repository secrets.

Changelog generation is intentionally manual through the project
`release-changelog` Cursor skill. Release CI only validates the committed
result, so no Cursor API key is stored in GitHub.

## What is required when

| Secret group | `0.x` tags | `1.0.0`+ stable tags |
| --- | --- | --- |
| Tauri updater | required | required |
| macOS Developer ID + notarization | optional | required |
| Windows Authenticode | optional | required |
| `TRAFFIC_TOKEN` | optional | optional |

Omitting an optional group on a `0.x` tag publishes that platform unsigned and
adds a warning banner to the release notes. Release CI fails a `1.0.0` or later
stable tag that is missing either vendor-signing group.

## Tauri updater

The first release establishes the updater trust root. Back up the private key
and password offline; losing them means existing installations cannot verify
future updates.

- `TAURI_SIGNING_PRIVATE_KEY`: contents of the generated `.key` file
- `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`: the password used to generate it

Generate a keypair outside the repository so it can never be committed:

```bash
mkdir -p ~/.oniongate-release && chmod 700 ~/.oniongate-release
openssl rand -base64 33 | tr -d '\n=+/' > ~/.oniongate-release/updater.password
npm run tauri -- signer generate --ci --force \
  --password "$(cat ~/.oniongate-release/updater.password)" \
  --write-keys ~/.oniongate-release/updater.key

gh secret set TAURI_SIGNING_PRIVATE_KEY < ~/.oniongate-release/updater.key
gh secret set TAURI_SIGNING_PRIVATE_KEY_PASSWORD < ~/.oniongate-release/updater.password
```

Copy the contents of `~/.oniongate-release/updater.key.pub` into the `pubkey`
field of `src-tauri/tauri.conf.json`. That public key is the only part that
belongs in the repository.

## macOS Developer ID signing

This is required for normal GitHub-download behavior under Gatekeeper; it does
not require Mac App Store distribution.

1. In Apple Developer Certificates, create a **Developer ID Application**
   certificate using a CSR from Keychain Access.
2. Install the downloaded certificate in the login keychain.
3. In Keychain Access, export the certificate and private key as a password
   protected `.p12`.
4. Configure:
   - `APPLE_CERTIFICATE`: base64 of the `.p12`
   - `APPLE_CERTIFICATE_PASSWORD`: export password
   - `APPLE_SIGNING_IDENTITY`: output such as
     `Developer ID Application: Name (TEAMID)`
   - `APPLE_TEAM_ID`: ten-character Developer Team ID

Useful local commands:

```bash
base64 -i OnionGate-Developer-ID.p12 | pbcopy
security find-identity -v -p codesigning
```

## Apple notarization

In App Store Connect, open **Users and Access → Integrations → Keys**. Create a
key with Developer access, download its `.p8` file once, and record the key and
issuer IDs.

- `APPLE_API_KEY`: Key ID
- `APPLE_API_ISSUER`: Issuer ID
- `APPLE_API_KEY_BASE64`: base64 of the downloaded `.p8`

```bash
base64 -i AuthKey_KEYID.p8 | pbcopy
```

The release workflow writes the key to a mode-600 temporary runner file and
passes its path to Tauri. Apple notarization and stapling happen automatically.

## Windows

Stable Windows distribution requires an Authenticode certificate:

- `WINDOWS_CERTIFICATE`: base64 of an exportable `.pfx`
- `WINDOWS_CERTIFICATE_PASSWORD`: `.pfx` password

Without these secrets, `0.x` tags publish a clearly labeled unsigned Windows
installer. Stable `1.0.0` or later tags fail.

## README clone badge

GitHub exposes release downloads publicly but keeps clone traffic private. The
daily docs workflow uses:

- `TRAFFIC_TOKEN`: a fine-grained PAT limited to this repository with
  **Administration: Read**.

Only the aggregate 14-day clone count is published. The default `GITHUB_TOKEN`
cannot access the Traffic API.

## Privileged helper (bundling + signing)

OnionGate includes a privileged-helper binary (`oniongate-helper`) so that,
after a single install prompt, its current typed kill-switch operations can run
without re-prompting. TUN, proxy, and hardening still use their normal elevation
paths. The app resolves the helper next to its own executable. Normal
development does not declare it as an `externalBin`; release CI builds the
target-specific helper and applies `src-tauri/tauri.release.conf.json` only to
distributable bundles. Tauri then signs the nested executable with the same
configured platform identity as the application.

Before publishing a draft, inspect the bundle:

1. macOS: `Contents/MacOS/oniongate-helper` exists and the outer app passes
   `codesign --verify --deep --strict`. launchd expects the helper and app to
   share the Team ID.
2. Linux: the helper is included beside the packaged executable and retains its
   executable mode.
3. Windows: `oniongate-helper.exe` is present and has the same valid
   Authenticode publisher as `OnionGate.exe`.

Hardening follow-up: split the helper into a minimal crate so the root daemon
does not carry the GUI dependency tree, and add client code-signature
verification (macOS audit token / Windows pipe ACL) before a stable release.
