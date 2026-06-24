# Release Readiness

Date: 2026-06-24

This document records the current release status for 900CRM and the manual
checks required before publishing a public release.

## Current Status

900CRM is not published as a packaged public desktop release yet.

Current CI is verification-only:

- it runs on `ubuntu-latest`;
- it verifies frontend linting, type checks, unit tests, production build, and
  browser smoke tests;
- it verifies Rust formatting, Clippy, workspace checks, and workspace tests;
- it does not build release installers;
- it does not sign, notarize, checksum, attach, or publish release artifacts;
- it does not run cross-platform packaging jobs for Windows, macOS, or Linux.

The current repository can be built and tested locally by contributors, but the
presence of source code, CI checks, or Tauri configuration is not a release
artifact.

## Manual Verification Checklist

Before a release candidate is tagged or published, maintainers should complete
the following checks from a clean checkout:

- [ ] Confirm the repository contains no local machine paths, private hostnames,
  secrets, tokens, or real customer data in source, docs, scripts, samples, or
  packaged assets.
- [ ] Run `npm ci`.
- [ ] Run `npm run lint`.
- [ ] Run `npm run check`.
- [ ] Run `npm run test`.
- [ ] Run `npm run build`.
- [ ] Run `npm run test:e2e`.
- [ ] Run `cargo fmt --all -- --check`.
- [ ] Run `cargo clippy --workspace -- -D warnings`.
- [ ] Run `cargo check --workspace`.
- [ ] Run `cargo test --workspace`.
- [ ] Manually smoke test the desktop app on every target platform that will
  receive an installer.
- [ ] Manually import the synthetic CSV samples under `samples/` for contacts
  and organizations.
- [ ] Manually create, edit, search, export, back up, validate, and restore a
  small local dataset.
- [ ] Confirm the privacy, import/export, backup/restore, MCP readiness, README,
  and changelog docs still describe only implemented behavior.
- [ ] Confirm the release notes list known gaps and deferred systems clearly.

## Required Future Release Artifacts

A public desktop release should include, at minimum:

- Windows installer: `.msi` or `.exe`.
- macOS disk image: `.dmg`.
- Linux package: `.deb`.
- Linux portable image: `.AppImage`.
- Checksums for every uploaded artifact.
- Software bill of materials (SBOM) for release contents.
- Release notes with known limitations and verification scope.
- License and third-party notice material where required by dependencies.
- Platform-specific signing and notarization evidence where applicable.

## Not Yet Implemented

The following release systems are intentionally not implemented in the current
repository state:

- automated release packaging workflow;
- automated Windows installer build;
- automated macOS DMG build;
- automated Linux `.deb` or `.AppImage` build;
- artifact signing;
- macOS notarization;
- checksum generation and publishing;
- SBOM generation and publishing;
- GitHub release creation or attachment automation;
- upgrade/update channel;
- release telemetry or crash reporting.

These gaps are release-engineering work for a future sprint. They are not app
runtime behavior, schema behavior, MCP behavior, AI behavior, or sync-server
behavior.

## Sample Data

The repository includes small synthetic CSV samples for manual import smoke
testing:

- `samples/contacts.csv`
- `samples/organizations.csv`

The sample rows are fake, culturally neutral, and not intended to represent real
people or organizations. They use the current documented CSV fields in
[Import and Export](IMPORT_EXPORT.md).
