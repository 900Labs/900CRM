# Sprint Ledger

| Sprint | Date (UTC) | Branch | Scope | Status |
|---|---|---|---|---|
| 001 | 2026-03-05 | `sprint-001-stabilization-ship-readiness` | Build blockers, IPC contract alignment, route/import-export/settings alignment, CI/smoke validation | Completed |
| 002 | 2026-03-05 | `sprint-002-warning-hardening` | Warning hardening: a11y semantics, reactive refs, CSS selector/media cleanup, Rust unused import cleanup, re-validation | Completed |
| 003 | 2026-03-05 | `sprint-003-global-modal-create-flows` | Global modal host for add-contact/add-deal/add-activity flows with context prefill and layout wiring | Completed |
| 004 | 2026-03-05 | `sprint-004-custom-fields-foundation` | Custom field storage/IPC foundation, frontend API wrapper, and lightweight mission guardrail checklist | Completed |
| 005 | 2026-03-05 | `sprint-005-custom-fields-ui` | Custom fields UI integration for contact/deal/activity create-edit flows using reusable dynamic input renderer | Completed |
| 006 | 2026-03-05 | `sprint-006-reports-metrics-backend` | Reporting metrics backend for pipeline conversion/activity funnels, IPC endpoints, and analytics index migration hardening | Completed |
| 007 | 2026-03-05 | `sprint-007-reports-dashboard-ui` | Dashboard integration for pipeline conversion/activity funnel reports with localized labels and lightweight visualization cards | Completed |
| 008 | 2026-03-05 | `sprint-008-custom-field-filters` | Custom-field filter UX for contacts/pipeline/activities with backend contact filtering and lightweight value-index lookups | Completed |
| 009 | 2026-03-05 | `sprint-009-desktop-notification-reminders` | Desktop reminder notifications for upcoming activities with persisted settings and lightweight polling | Completed |
| 010 | 2026-03-05 | `sprint-010-ptbr-vietnamese-translations` | Portuguese (Brazil) and Vietnamese full-locale support with lazy-load i18n registration and docs alignment | Completed |
| 011 | 2026-03-05 | `sprint-011-multi-currency-display` | Multi-currency-safe dashboard/pipeline value display with grouped currency totals and normalized currency handling | Completed |
| 012 | 2026-03-05 | `sprint-012-email-integration-optional` | Optional IMAP/SMTP settings with lightweight endpoint tests and local `mailto` compose action | Completed |
| 013 | 2026-03-05 | `sprint-013-hausa-bengali-translations` | Hausa and Bengali full-locale support with i18n lazy-loader wiring and roadmap/doc closure | Completed |
| 014 | 2026-06-23 | `codex/backup-ui-surface-preserve` | Backup API wrappers, Settings Data Management controls, explicit restore confirmation, tests, and backup/restore docs | Completed |
| 015 | 2026-06-24 | `codex/organization-ui-surface-preserve` | Organization frontend API, store, route, navigation, i18n, and focused invoke-mapping tests while preserving contact organization text | Completed |
| 016 | 2026-06-24 | `codex/notes-tags-core-surface-preserve` | Generic notes/tags `crm-core` services, Tauri commands, frontend API wrappers, compatibility mirroring, and focused tests without route/UI work | Completed |
| 017 | 2026-06-24 | `codex/notes-tags-contract-hardening` | Notes/tags contract hardening for idempotent tag link audit/sync semantics and explicit tag color reset behavior without UI work | Completed |
| 018 | 2026-06-24 | `codex/notes-tags-ui-surface` | Reusable generic notes/tags Svelte panels integrated for contacts and organizations while preserving legacy contact notes/tags | Completed |
| 019 | 2026-06-24 | `codex/deal-relationships-core-surface` | Deal organization and deal contact core/API foundation with schema v7, legacy contact mirror preservation, and no Pipeline/Deal UI changes | Completed |
| 020 | 2026-06-24 | `codex/deal-relationships-ui-surface` | Deal create modal organization/contact selectors and pipeline card relationship labels using existing frontend APIs without backend changes | Completed |
| 021 | 2026-06-24 | `codex/activity-relationships-core-surface` | Activity contact, organization, and deal core/API relationship foundation with schema v8, legacy contact/deal mirror preservation, and no activity relationship UI | Completed |
| 022 | 2026-06-24 | `codex/activity-relationships-ui-surface` | Activity create controls and activity list relationship labels for contact, organization, and deal links using existing frontend APIs without backend changes | Completed |
| 023 | 2026-06-24 | `codex/global-search-core-surface` | Global search core/Tauri/frontend API surface for contacts, organizations, deals, activities, notes, and tags without SearchBar UI changes | Completed |
| 024 | 2026-06-24 | `codex/global-search-ui-surface` | Visible SearchBar integration with global search results across contacts, organizations, deals, activities, notes, and tags without backend changes | Completed |
| 025 | 2026-06-24 | `codex/import-export-api-organization-csv-foundation` | Organization CSV import/export foundation and typed frontend Import/Export API wrappers without mapping wizard or duplicate-warning UI | Completed |
| 026 | 2026-06-24 | `codex/import-preflight-duplicate-detection-foundation` | Read-only CSV import preflight duplicate warning foundation for contacts and organizations across core, Tauri, and frontend API layers without wizard UI changes | Completed |
| 027 | 2026-06-24 | `codex/import-field-mapping-core-surface` | Field-mapped CSV import/preflight core, Tauri, and frontend API foundation for contacts and organizations without Import Wizard UI | Completed |
| 028 | 2026-06-24 | `codex/import-wizard-ui-surface` | Visible contact and organization import wizard UI with CSV preview, field mapping, duplicate warnings, explicit confirmation, and summary while preserving deal legacy import/export | Completed |
| 029 | 2026-06-24 | `codex/normalization-migration-readiness-current-main` | Read-only normalization migration readiness preflight for legacy organization contacts, split invalid organization links, backup baseline status, and focused Rust tests without destructive migration | Completed |
| 030 | 2026-06-24 | `codex/contact-duplicate-merge-ui-surface` | Contact duplicate warning and merge UI using read-only email/phone candidate pairs plus existing audited contact merge behavior | Completed |
| 031 | 2026-06-24 | `codex/audit-pending-actions-api-surface` | Read-only audit log and pending proposed-action Tauri commands plus typed frontend API wrappers/tests without route or UI work | Completed |
| 032 | 2026-06-24 | `codex/audit-pending-actions-ui-surface` | Visible read-only Audit Log and Pending Actions UI routes wired into hash routing/sidebar using Sprint 031 frontend API wrappers | Completed |
| 033 | 2026-06-24 | `codex/external-clients-api-surface` | Disabled external client placeholder Tauri commands plus typed frontend API wrappers/tests without UI, grants, tokens, MCP, AI, or sync server behavior | Completed |
| 034 | 2026-06-24 | `codex/external-clients-ui-surface` | Settings Integrations surface for listing and creating disabled external client placeholders through Sprint 033 frontend API wrappers | Completed |
| 035 | 2026-06-24 | `codex/proposed-actions-decision-api-surface` | Pending-only proposed-action approve/reject core, Tauri, and frontend API foundation with audit entries and no execution/UI/MCP behavior | Completed |
| 036 | 2026-06-24 | `codex/proposed-actions-decision-ui-surface` | `/pending-actions` approve/reject decision controls using Sprint 035 frontend API wrappers with pending-only refresh/removal and no execution behavior | Completed |
| 037 | 2026-06-24 | `codex/external-client-permissions-core-surface` | External-client permission row storage and core evaluation for read/draft access with proposed-action draft gating and no UI/Tauri/MCP behavior | Completed |
| 038 | 2026-06-24 | `codex/external-client-permissions-api-surface` | Thin Tauri commands and typed frontend API wrappers/tests for external-client permission listing, upsert, and read/draft evaluation without UI work | Completed |
| 039 | 2026-06-24 | `codex/mcp-readiness-docs-baseline` | Documentation-only MCP readiness baseline covering placeholder status, active readiness surfaces, non-goals, security gates, and future acceptance checklist | Completed |
| 040 | 2026-06-24 | `codex/required-docs-baseline` | Documentation-only required public baseline for data model, import/export, and privacy docs with narrow README/architecture links | Completed |
| 041 | 2026-06-24 | `codex/verification-gates-hardening` | Local verification gate hardening for ESLint 9 flat config and existing Rust Clippy warnings without product behavior changes | Completed |
