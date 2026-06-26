# Release Readiness

Date: 2026-06-24

This document records the current release status for 900CRM and the manual
checks required before publishing a public release.

## Current Status

900CRM is not published as a packaged public desktop release yet.

Current CI is verification-only, and release packaging is intentionally
separate. Manual release packaging now has an automated Ubuntu preflight gate
before any platform packaging starts:

- it runs on `ubuntu-latest`;
- it verifies the public-release guardrail scan, frontend linting, type checks,
  unit tests, production build, and browser smoke tests;
- it verifies Rust formatting, Clippy, workspace checks, and workspace tests;
- package generation does not start unless that preflight passes;
- normal CI still does not build or publish release installers.

Manual release packaging now lives in
[`.github/workflows/release.yml`](../.github/workflows/release.yml). It must be
started with `workflow_dispatch`, runs the preflight gate, then builds Windows,
macOS, and Linux Tauri bundles, uploads workflow artifacts, and generates
per-platform SHA-256 checksums, release metadata, and an SPDX-shaped dependency
inventory. The workflow does not publish a GitHub Release unless
`publish_github_release` is explicitly enabled for a matching `v`-prefixed tag
ref.

The current repository can be built and tested locally by contributors, but the
presence of source code, CI checks, or Tauri configuration is not a release
artifact.

## Verification Checklist

Before a release candidate is tagged or published, maintainers should complete
the following checks from a clean checkout. The command-only gates are enforced
automatically by the manual release packaging workflow preflight before package
artifacts are built; the platform, data, signing, and release-note checks remain
manual.

- [ ] Confirm the repository contains no local machine paths, private hostnames,
  secrets, tokens, or real customer data in source, docs, scripts, samples, or
  packaged assets.
- [ ] Run `npm ci`.
- [ ] Run `npm run check:release-guardrails`.
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


### Public Release Guardrail Scan

`npm run check:release-guardrails` runs a deterministic scan over tracked
repository text files. It catches high-confidence local-machine path leaks,
private/internal host URLs, and literal credential patterns before a release or
PR is accepted. The scan intentionally does not treat ordinary variable names
such as `token`, placeholder values, empty passwords, or generic `/tmp/...` test
paths as release blockers.

This automated check is a guardrail, not a substitute for manual review.
Maintainers must still confirm that samples, screenshots, docs, and packaged
assets do not contain real customer data or other sensitive material that a
pattern scan cannot reliably identify.

## Release Artifacts and Metadata

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

## Manual Packaging Workflow

Maintainers can create unsigned release-candidate packages from a clean checkout
by dispatching the `Manual Release Packaging` workflow with:

- `release_version`: the version label recorded in artifact metadata.
- `release_title`: optional human-readable release title.
- `release_tag`: required only when attaching artifacts to a GitHub Release.
- `publish_github_release`: default `false`; set to `true` only from the
  matching `refs/tags/v...` workflow ref.
- `release_draft`: default `true` for guarded GitHub Release creation.

The workflow produces two artifact groups for each platform:

- desktop packages built by Tauri under
  `target/release/bundle`;
- release metadata from `scripts/generate-release-manifest.mjs`, including
  `*-SHA256SUMS.txt`, `*-release-metadata.json`, and `*-sbom.spdx.json`.

The packaging matrix depends on the `preflight` job. That job installs the same
Ubuntu system dependencies used by CI, runs `npm ci`, checks public release
guardrails, runs the documented frontend and browser smoke gates, and runs the
documented Rust formatting, lint, check, and test gates. A failing preflight
blocks all package jobs before any release artifacts are generated.

The manifest script is also locally runnable for validation:

```bash
node scripts/generate-release-manifest.mjs --help
node scripts/generate-release-manifest.mjs --sample --out-dir /tmp/900crm-release-sample --platform local
```

If real release artifacts are not present and `--sample` is not used, the script
fails with a message naming the missing artifact directory or expected package
suffixes.

## Still Not Implemented

The following release systems remain intentionally not implemented in the
current repository state:

- artifact signing;
- macOS notarization;
- automatic publishing by default;
- release attachment automation without an explicit `publish_github_release`
  dispatch input and matching `refs/tags/v...` guard;
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
