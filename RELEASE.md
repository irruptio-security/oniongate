# Release process

1. Update pinned application and sidecar versions.
2. Verify `scripts/dependencies.sha256` from two independent upstream sources.
3. Run `npm run check` and the full Rust test suite.
4. Build from a clean tagged checkout in GitHub Actions.
5. Sign and notarize the macOS application and all nested sidecars.
6. Produce Linux packages; sign release checksums.
7. Generate CycloneDX SBOMs for Rust, npm, and bundled sidecars.
8. Publish `SHA256SUMS`, signatures, SBOMs, licenses, and corresponding-source
   links with the release.
9. Test a quarantined download on a fresh macOS VM and a clean Linux VM.
10. Publish the updater manifest only after artifact verification.

Release workflows require repository secrets described in
`.github/RELEASE_SECRETS.md`. Missing macOS or updater credentials fail the
release. Missing Windows credentials produce an explicitly experimental
unsigned artifact that must never be labeled stable.
