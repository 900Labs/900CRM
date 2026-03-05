# Changelog

All notable changes to 900CRM are documented in this file.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [Unreleased]

### Added — 2026-03-05
- Added a global modal host component that centralizes create flows for contacts, deals, and activities from any route.
- Wired root layout rendering for global modal support so dashboard, pipeline, contacts, and detail-page quick actions execute consistently.
- Added context-aware modal prefill for linked entity creation (`contactId`, `dealId`) and stage-aware deal creation.
- Added custom field foundation APIs:
  - Rust storage layer for custom field definitions and values.
  - Tauri IPC commands for listing/creating/updating/deleting definitions and setting/listing values.
  - Frontend API wrapper (`src/lib/api/customFields.ts`) for typed invoke integration.
- Added reusable dynamic custom-field renderer component for create/edit forms (`src/lib/components/CustomFieldInputs.svelte`).
- Added custom field UI integration for contact/deal/activity create flows in the global modal host.
- Added custom field UI integration for contact detail edit flow, including value loading, editing, save, and cancel reset behavior.
- Added reporting backend contracts for analytics:
  - Pipeline conversion report endpoint (`get_pipeline_conversion_report`).
  - Activity funnel report endpoint (`get_activity_funnel_report`).
  - Typed frontend API wrapper (`src/lib/api/reports.ts`).
- Added dashboard report UI integration for analytics:
  - Pipeline conversion and activity funnel cards on the dashboard.
  - Lightweight stage/type bar visualization with localized labels.
  - Graceful partial-failure fallback so core dashboard stats still load if reports are unavailable.
- Added custom-field filter UX for primary operational views:
  - Contacts list supports backend-filtered custom-field searching (pagination-safe).
  - Pipeline and Activities support lightweight custom-field filtering via local value indexes.
  - Added bulk custom-field value command for per-entity-type filter lookups.
- Added desktop reminder notifications for upcoming activities:
  - Added reminder polling service with at-most-once notifications per activity due timestamp.
  - Added settings for desktop reminders (`notifications_enabled`, `reminder_lead_minutes`).
  - Added schema migration v3 to backfill reminder setting defaults for existing installs.
- Added complete Portuguese (Brazilian) and Vietnamese locale support:
  - Added `src/lib/i18n/pt.json` and `src/lib/i18n/vi.json`.
  - Registered `pt` and `vi` in `availableLocales` and lazy locale loading.
- Added multi-currency display support across dashboard and pipeline:
  - Added grouped pipeline currency totals in dashboard stats (`pipeline_value_by_currency`).
  - Updated dashboard KPI rendering to handle mixed-currency pipelines without forcing a single-currency format.
  - Updated pipeline stage totals to show per-currency totals for mixed-currency columns.
  - Added currency normalization helpers and stricter deal-currency input normalization.
- Added optional email integration (IMAP/SMTP):
  - Added email integration settings for SMTP/IMAP endpoints and local compose workflow.
  - Added lightweight SMTP/IMAP endpoint reachability test command (`test_email_server_connection`).
  - Added schema migration v4 to seed email integration setting defaults for existing installs.
  - Added contact detail action to open local mail composer via `mailto:`.
- Added complete Hausa and Bengali locale support:
  - Added `src/lib/i18n/ha.json` and `src/lib/i18n/bn.json`.
  - Registered `ha` and `bn` in `availableLocales` and lazy locale loading.
- Added schema migration v2 analytics indexes to keep report queries efficient on constrained hardware.
- Added a lightweight open-source and low-resource guardrail checklist and wired it into the PR template/workflow.

### Fixed — 2026-03-05
- Resolved Svelte/Tauri build blockers (invalid event attributes, modal snippet typing, SVG attribute typing, and Tauri config schema compatibility).
- Unified frontend IPC wrappers with backend command contracts (snake_case argument names, response mapping, and stage/type/status normalization).
- Added missing sync IPC commands (`get_sync_status`, `trigger_sync`) and activity revert command (`mark_activity_incomplete`).
- Aligned hash-route rendering so `/contacts`, `/contacts/:id`, `/pipeline`, `/activities`, and `/settings` render correctly from the root page.
- Aligned import/export flows with implemented backend CSV commands (`import_*_csv`, `export_*_csv`) and file-path based operations.
- Removed remaining Svelte/Tauri warning classes (a11y semantics, rune reactivity refs, dark-theme selector scoping, CSS media-token syntax, and Rust unused imports).
- Verified clean warning baseline: `npm run check` now reports 0 errors and 0 warnings; `cargo check` is warning-free.
- Corrected deal modal translation key usage from `deals.dealName` to existing locale key `deals.name`.
- Fixed TypeScript nullability issues in contact-detail quick actions for add-deal/add-activity modal context opening.
- Fixed contact-detail save/cancel consistency by including custom field state in dirty/reset/save handling.

---

## [1.0.0] — March 2026

### Summary

The initial release of 900CRM — a free, open-source, offline-first desktop CRM built with Tauri v2 (Rust), Svelte 5, and SQLite. Designed for small businesses, NGOs, and sales teams in developing nations who need reliable CRM tooling without cloud dependencies, per-seat subscription costs, or reliable internet access.

---

### Added

#### Contacts
- Create, view, update, and delete contacts (people and organizations)
- Fields: name, email, phone, address, company, job title, website, notes
- Tag contacts with custom labels for categorization and filtering
- Link contacts to deals and activities
- Contact detail panel with full activity history
- Bulk import contacts from CSV with field mapping interface
- Export contacts to CSV

#### Pipeline (Deals)
- Visual kanban board with drag-and-drop deal cards
- Default pipeline stages: Lead, Qualified, Proposal, Negotiation, Closed Won, Closed Lost
- Rename and reorder pipeline stages
- Deal fields: name, value, currency, expected close date, contact link, notes
- Deal detail panel showing linked contact, all activities, and stage history
- Pipeline summary: total value by stage, count of deals per stage
- Export deals to CSV

#### Activities
- Create activities of type: Task, Call, Meeting, Email, Note
- Link activities to contacts and/or deals
- Set due dates and mark activities complete
- Activity feed sorted by due date with overdue highlighting
- Filter activities by type, status (open/complete), contact, or deal
- Bulk mark activities complete
- Export activities to CSV

#### Dashboard
- At-a-glance metrics panel on application launch
- Pipeline total value (sum of all open deals)
- Deals by stage — visual breakdown
- Activities due today and overdue count
- Recent contacts added (last 7 days)
- Upcoming activities (next 7 days)
- Pipeline win rate (closed won / total closed)

#### Search
- Full-text search across contacts, deals, and activities simultaneously
- Instant results as you type (debounced, no latency)
- Results grouped by entity type with contextual previews
- Keyboard navigation through results
- SQLite FTS5 index for fast offline full-text search

#### Import / Export
- CSV import for contacts: supports standard exports from Salesforce, HubSpot, Google Contacts, and generic CSV
- Field mapping interface for non-standard column names
- Duplicate detection on import (by email address)
- Export any entity type (contacts, deals, activities) to CSV
- All import/export operations run entirely offline

#### Internationalization
- Full i18n support with language switching in settings (no restart required)
- English (en) — 100% complete (base language)
- French (fr) — 100% complete
- Spanish (es) — 100% complete
- Arabic (ar) — 100% complete with full RTL layout
- Swahili (sw) — 100% complete
- Hindi (hi) — 85% complete
- RTL (right-to-left) layout support for Arabic and future RTL languages

#### Settings
- Language selection
- Date format preference (DD/MM/YYYY, MM/DD/YYYY, YYYY-MM-DD)
- Currency display preferences
- Theme (light, dark, system default)
- Data directory location (for backup purposes)

#### Performance and Compatibility
- Application startup in under 500ms on typical hardware
- Handles 10,000+ contacts without performance degradation
- Installer under 8 MB on all platforms
- Memory usage under 80 MB at idle
- Optimized for hardware from 2015 onward (dual-core, 4 GB RAM)

#### Platform Support
- Windows 10 and later (x64) — `.msi` installer
- macOS 11 (Big Sur) and later — Intel x64 and Apple Silicon ARM64 — `.dmg`
- Linux — `.deb` (Debian/Ubuntu 20.04+) and `.AppImage` (universal)

#### Developer / Community
- Full source code under Apache License 2.0
- Rust backend with complete `///` doc comments on all public APIs
- Svelte 5 frontend with typed props and component documentation headers
- GitHub Actions CI pipeline running on Ubuntu, Windows, and macOS
- Automated release builds on version tags
- Community issue templates (bug report, feature request)
- Contributor guide ([CONTRIBUTING.md](CONTRIBUTING.md))
- Architecture guide ([ARCHITECTURE.md](ARCHITECTURE.md))
- Plugin system architecture defined (implementation planned for v2.0)

---

## Version History Format

This project uses [Semantic Versioning](https://semver.org/):

- **MAJOR** version (1.x.x → 2.x.x): incompatible changes such as database schema migrations that cannot be done automatically, or removal of major features
- **MINOR** version (x.1.x → x.2.x): new features added in a backwards-compatible manner
- **PATCH** version (x.x.1 → x.x.2): backwards-compatible bug fixes

Pre-release versions use a suffix: `1.1.0-alpha.1`, `1.1.0-beta.2`, `1.1.0-rc.1`.

[Unreleased]: https://github.com/900-labs/900crm/compare/v1.0.0...HEAD
[1.0.0]: https://github.com/900-labs/900crm/releases/tag/v1.0.0
