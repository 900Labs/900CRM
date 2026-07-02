# 900CRM Documentation Index

This directory contains two kinds of documentation:

- Current product truth: documents that describe the app, data model, release
  status, privacy posture, import/export behavior, backup/restore behavior, and
  accepted MCP boundary as they exist now.
- Historical sprint evidence: `sprint_*.md` files and `sprint_ledger.md`.
  These are useful audit records, but they should not be the first place a
  contributor looks for the current state.

## Start Here

| Need | Document |
|---|---|
| Product status, feature depth, and competitive gaps | [Product Review and Competitive Benchmark](PRODUCT_REVIEW_AND_BENCHMARK.md) |
| Alpha source/release readiness | [Alpha Release Readiness Audit](ALPHA_READINESS.md) |
| Release packaging and artifact requirements | [Release Readiness](RELEASE.md) |
| Local schema, migrations, and data boundaries | [Data Model](DATA_MODEL.md) |
| Import/export formats and limitations | [Import and Export](IMPORT_EXPORT.md) |
| Backup and destructive restore workflow | [Backup and Restore](BACKUP_RESTORE.md) |
| Privacy, local data, and network behavior | [Privacy](PRIVACY.md) |
| MCP current scope and deferred future scope | [MCP Readiness Baseline](MCP_READINESS.md) |
| Public release hygiene checklist | [Open Source Guardrail Checklist](OPEN_SOURCE_GUARDRAIL_CHECKLIST.md) |

## Sprint Records

Historical sprint notes are intentionally kept for auditability. They record
what each narrow sprint changed, what it did not change, and which verification
commands ran at the time. Prefer the current truth documents above when making
product, release, or roadmap decisions.

Use [Sprint Ledger](sprint_ledger.md) when you need a chronological map of
completed sprint branches.

## Documentation Rules

- Do not make README or release docs claim installable public artifacts until
  package artifacts exist and have been smoke-tested.
- Keep future MCP network server, real sync transport, built-in AI behavior,
  signing, notarization, telemetry, and auto-update work clearly labeled as
  deferred unless an implementation sprint actually lands it.
- When product behavior changes, update the durable current-state document
  first, then add a sprint note if the work needs audit evidence.
- Do not put local machine paths, private hostnames, secrets, tokens, or real
  customer data in public docs.
