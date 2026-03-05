# Changelog

All notable changes to 900CRM are documented in this file.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [Unreleased]

### In Progress
- Custom fields on contacts, deals, and activities — planned for v1.1.0
- Desktop notifications and reminders for upcoming activities — planned for v1.1.0
- Pipeline conversion reports and activity funnel analytics — planned for v1.1.0
- Portuguese (Brazilian) and Vietnamese translations — planned for v1.1.0

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
