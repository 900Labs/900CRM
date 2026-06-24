# 900CRM Architecture Guide

This document describes the technical architecture of 900CRM for contributors. It covers the high-level system design, module structure, data flow, offline-first principles, sync protocol, database schema, and key design decisions.

If you are new to the codebase, read this document before diving into the source. It will help you understand *why* things are built the way they are.

---

## Table of Contents

- [High-Level Overview](#high-level-overview)
- [Module Map](#module-map)
- [Frontend Architecture](#frontend-architecture)
- [Backend Architecture](#backend-architecture)
- [Data Flow](#data-flow)
- [Offline-First Design](#offline-first-design)
- [Sync Protocol](#sync-protocol)
- [Database Schema](#database-schema)
- [Extension Points](#extension-points)
- [MCP Readiness](#mcp-readiness)
- [Key Design Decisions](#key-design-decisions)

---

## High-Level Overview

900CRM is a two-process desktop application built on [Tauri v2](https://tauri.app/). The two processes are:

1. **Rust backend** — a Tauri shell in `apps/desktop/src-tauri` plus shared CRM logic in `crates/crm-core`. Handles all data operations, business logic, file I/O, and import/export.
2. **WebView frontend** — a Svelte 5 application rendered in the system WebView. Handles all UI rendering and user interaction. Cannot access the OS directly.

The two processes communicate via **Tauri IPC**: the frontend calls named commands, the backend executes them, and returns results as serialized JSON.

```
┌──────────────────────────────────────────────────────────────────┐
│                         Operating System                          │
│                                                                  │
│  ┌──────────────────────────┐    ┌────────────────────────────┐  │
│  │      Rust Backend         │    │    WebView (Frontend)       │  │
│  │ apps/desktop/src-tauri    │◄──►│    apps/desktop/src         │  │
│  │ crates/crm-core           │    │    Svelte 5 / TypeScript   │  │
│  │                          │    │                            │  │
│  │  ┌────────────────────┐  │    │  ┌──────────────────────┐  │  │
│  │  │   Tauri Commands   │  │    │  │   Routes / Pages     │  │  │
│  │  │  (IPC handlers)    │  │    │  │  /dashboard          │  │  │
│  │  └────────┬───────────┘  │    │  │  /contacts           │  │  │
│  │           │              │    │  │  /pipeline           │  │  │
│  │  ┌────────▼───────────┐  │    │  │  /activities         │  │  │
│  │  │    CRM Engine       │  │    │  └──────────────────────┘  │  │
│  │  │  (business logic)   │  │    │                            │  │
│  │  └────────┬───────────┘  │    │  ┌──────────────────────┐  │  │
│  │           │              │    │  │   Components          │  │  │
│  │  ┌────────▼───────────┐  │    │  │  contacts/ pipeline/ │  │  │
│  │  │   Storage Layer     │  │    │  │  activities/dashboard│  │  │
│  │  │ (rusqlite + sync)   │  │    │  └──────────────────────┘  │  │
│  │  └────────┬───────────┘  │    │                            │  │
│  └───────────┼──────────────┘    │  ┌──────────────────────┐  │  │
│              │                   │  │   Stores (rune state) │  │  │
│              │                   │  └──────────────────────┘  │  │
│    ┌─────────▼──────────┐        │                            │  │
│    │   SQLite Database   │        │  ┌──────────────────────┐  │  │
│    │  (~/.local/share/   │        │  │   API layer           │  │  │
│    │   900CRM/data.db)   │        │  │  (invoke wrappers)   │  │  │
│    └────────────────────┘        └──┴──────────────────────┴──┘  │
│                                                                  │
│                    Tauri IPC bridge (JSON over stdin/stdout)      │
└──────────────────────────────────────────────────────────────────┘
```

---

## Module Map

### Tauri Shell (`apps/desktop/src-tauri/src/`)

```
apps/desktop/src-tauri/src/
├── main.rs                    Native entry point.
├── lib.rs                     Configures and launches the Tauri app.
├── state.rs                   Managed application state shared by commands.
│
├── commands/                  Tauri IPC command handlers.
│   │                          These are thin wrappers: validate input, call
│   │                          crm-core services, return serialized result or error.
│   │                          No business logic lives here.
│   │
│   ├── contact_commands.rs    create_contact, get_contact, list_contacts,
│   │                          update_contact, delete_contact, search_contacts
│   │
│   ├── deal_commands.rs       create_deal, get_deal, list_deals, update_deal,
│   │                          delete_deal, move_deal_to_stage, list_pipeline_stages,
│   │                          create_pipeline_stage, update_pipeline_stage
│   │
│   ├── activity_commands.rs   create_activity, get_activity, list_activities,
│   │                          update_activity, delete_activity, complete_activity
│   │
│   ├── import_export.rs       import_contacts_csv, export_contacts_csv,
│   │                          import_deals_csv, export_deals_csv,
│   │                          export_activities_csv, get_import_preview
│   │
│   ├── dashboard_commands.rs  Dashboard metrics and summaries
│   ├── report_commands.rs     Reporting command handlers
│   ├── custom_field_commands.rs
│   ├── organization_commands.rs
│   ├── email_commands.rs
│   ├── sync_commands.rs
│   ├── backup_commands.rs
│   └── settings_commands.rs   get_settings, update_settings
```

### CRM Core (`crates/crm-core/src/`)

```
crates/crm-core/src/
├── crm_engine/                Business rules, fully decoupled from Tauri.
│   ├── contacts.rs            Contact validation and relationship logic
│   ├── deals.rs               Deal rules and pipeline calculations
│   ├── activities.rs          Activity scheduling and completion tracking
│   ├── pipeline.rs            Pipeline stage rules
│   └── search.rs              Search query building and ranking
│
├── storage/                   SQLite access layer. All SQL lives here.
│   ├── db.rs                  Database opening and migration setup
│   ├── contacts.rs            Contact persistence
│   ├── deals.rs               Deal persistence
│   ├── activities.rs          Activity persistence
│   ├── sync.rs                Changelog and sync persistence
│   ├── settings.rs            Settings persistence
│   └── search.rs              FTS persistence
│
├── domain/                    Domain models shared by services and storage.
├── services/                  Application services used by Tauri commands.
├── search/                    Search support modules.
├── import_export/             CSV import/export support.
├── audit/                     Audit support.
├── permissions/               Permission helpers.
└── utils/                     Shared utility modules.
```

### Rust Placeholders (`crates/crm-mcp/`, `crates/crm-sdk/`)

```
crates/crm-mcp/src/main.rs     Placeholder for future optional MCP integration.
crates/crm-sdk/src/lib.rs      Placeholder for future SDK exports.
```

The current MCP package is not implemented and is not started by the desktop app or `crm-core`.

### Frontend (`apps/desktop/src/`)

```
apps/desktop/src/
├── routes/                    SvelteKit routes and route-level components
│   ├── +layout.svelte         App shell: persistent sidebar navigation
│   ├── +page.svelte           Default route
│   ├── Dashboard.svelte       Dashboard metrics and recent activity feed
│   ├── Contacts.svelte        Contacts list with search and filter
│   ├── ContactDetail.svelte   Contact detail panel with activity timeline
│   ├── Pipeline.svelte        Kanban board — deals grouped by stage
│   ├── Activities.svelte      Activity feed with filters
│   └── Settings.svelte        Language, theme, date format, data location
│
├── lib/
│   ├── components/            Reusable Svelte components
│   ├── stores/                Svelte 5 rune-based reactive state
│   ├── api/                   Typed wrappers around Tauri invoke calls
│   ├── i18n/                  Translation files and i18n loader
│   ├── services/              Frontend service helpers
│   └── utils/                 Shared helper functions
│
├── app.css                    Global styles and design tokens
├── app.d.ts
└── app.html
```

---

## Frontend Architecture

### Route Structure

900CRM uses SvelteKit's file-system routing. Each route corresponds to a top-level navigation item. The persistent layout (`+layout.svelte`) wraps all routes and provides the sidebar navigation and top search bar.

### Component Patterns

All components follow consistent patterns:

1. **Documentation header** — every `.svelte` file starts with a `@component` JSDoc block describing its purpose and props.
2. **Typed props** — props are declared with a TypeScript `interface` and `$props()`.
3. **Rune-based state** — internal state uses `$state()`. Derived values use `$derived()`. Side effects use `$effect()`.
4. **No direct API calls** — components never call `invoke()` directly. They call store methods or API functions, which call `invoke()`.

### State Management

State is managed in store modules (`apps/desktop/src/lib/stores/`). Each store exports reactive `$state` variables and mutating functions. Components subscribe by importing the store.

```typescript
// apps/desktop/src/lib/stores/contacts.ts
let contacts = $state<Contact[]>([]);
let loading = $state(false);

export async function loadContacts() {
  loading = true;
  contacts = await api.listContacts();
  loading = false;
}

export { contacts, loading };
```

### API Layer

The `apps/desktop/src/lib/api/` layer provides typed wrappers around Tauri's `invoke()` function. This separation means:
- Components never depend on the string names of Tauri commands
- TypeScript enforces the correct argument and return types
- The API layer can be mocked in unit tests without mocking Tauri

```typescript
// apps/desktop/src/lib/api/contacts.ts
import { invoke } from '@tauri-apps/api/core';
import type { Contact, NewContact, UpdateContact } from '$lib/types';

export async function createContact(data: NewContact): Promise<Contact> {
  return await invoke<Contact>('create_contact', { data });
}

export async function listContacts(filter?: ContactFilter): Promise<Contact[]> {
  return await invoke<Contact[]>('list_contacts', { filter });
}
```

### i18n

All user-facing strings use the `t()` function from the i18n module:

```svelte
<script lang="ts">
  import { t } from '$lib/i18n';
</script>

<h1>{t('dashboard.title')}</h1>
<button>{t('contacts.add')}</button>
```

Language switching is reactive and takes effect immediately without a page reload.

---

## Backend Architecture

### Command Handlers

Commands (`apps/desktop/src-tauri/src/commands/`) are the entry points for all frontend requests. They follow a strict pattern:

1. Deserialize input (Tauri handles JSON deserialization automatically)
2. Validate input (call `crm-core` validation functions)
3. Acquire a database connection from the managed connection pool
4. Call the appropriate `crm-core` service or engine function
5. Return the result (Tauri serializes to JSON automatically)

Commands contain no business logic. They are glue between the IPC layer and the engine.

### CRM Engine

The engine (`crates/crm-core/src/crm_engine/`) contains business logic. It is part of a pure Rust crate with no Tauri dependencies, which makes it unit-testable without spinning up a Tauri application.

The engine functions take a `&Connection` parameter (the SQLite connection) and the domain input, and return `Result<T, CrmError>`. They call storage queries to read and write data, apply business rules, and return domain objects.

### Storage Layer

The storage layer (`crates/crm-core/src/storage/`) owns SQL persistence. Every query is a Rust function with typed parameters and return types. This keeps SQL auditable in one place and isolates schema changes to the shared core crate.

---

## Data Flow

The complete path of a user action through the system:

```
User clicks "Add Contact"
         │
         ▼
[Svelte component]
ContactEditor.svelte → validates form fields locally
         │
         ▼
[API layer]
apps/desktop/src/lib/api/contacts.ts → createContact(formData)
         │
         ▼
[Tauri IPC bridge]
invoke('create_contact', { data: formData })
         │ (JSON serialized)
         ▼
[Rust command handler]
apps/desktop/src-tauri/src/commands/contact_commands.rs → create_contact(data, state)
  • Input deserialized from JSON
  • State (DB connection pool) acquired
         │
         ▼
[CRM Engine]
crates/crm-core/src/crm_engine/contacts.rs → create_contact(&conn, new_contact)
  • Validates required fields
  • Normalizes tags (lowercase, trim)
  • Checks for duplicate email
  • Generates UUID for new contact
         │
         ▼
[Storage layer]
crates/crm-core/src/storage/contacts.rs → insert_contact(&conn, contact)
  • Executes INSERT INTO contacts ...
  • Inserts associated tags into contact_tags
  • Inserts changelog entry (for sync)
         │
         ▼
[SQLite database]
data.db → record persisted
         │
         ▼
[Result bubbles back up]
Contact struct → serialized to JSON → returned to frontend
         │
         ▼
[API layer]
createContact() resolves with Contact
         │
         ▼
[Store]
contacts store → appends new contact to local $state
         │
         ▼
[UI]
ContactList re-renders reactively with new contact visible
```

---

## Offline-First Design

Offline-first means the application works correctly when there is no internet connection, and that network connectivity is **never a prerequisite** for any feature. This is the default operating mode, not a fallback.

### Core Principle: Local SQLite is the Source of Truth

All reads and writes go to the local SQLite database. There is no "optimistic UI" backed by a remote store. Every operation is committed to local storage before it is reflected in the UI.

### No Network Dependency in the Core

The Rust backend does not make any outbound network calls as part of normal operation. The Tauri `tauri.conf.json` has all network capabilities disabled by default:

```json
{
  "app": {
    "security": {
      "csp": "default-src 'self'"
    }
  }
}
```

### Optional Sync

Multi-device sync is an optional, user-initiated feature for teams. It does not change the offline-first guarantees:
- All writes always go to local SQLite first
- Sync pushes local changes to a sync server when online
- Sync pulls remote changes and merges them locally
- The app works identically whether sync is configured or not

See the [Sync Protocol](#sync-protocol) section for implementation details.

### File Size Rationale

The installer is under 8 MB because 900CRM uses the system's built-in WebView (Edge on Windows, WebKit on macOS, WebKitGTK on Linux) rather than bundling Chromium. This matters for users downloading over slow or metered mobile connections — common in the regions 900CRM is built for.

---

## Sync Protocol

> **Note:** The sync feature is fully designed but not yet implemented in v1.0. This section documents the planned architecture for v1.1+.

### Design Goals

1. Works without any sync configuration (single-device use is the default)
2. Self-hostable sync server — no vendor lock-in
3. Tolerates intermittent connectivity: sync when connected, work normally when not
4. Simple conflict resolution that is correct for CRM use cases

### Changelog Table

Every mutating operation on any entity writes a record to the `changelog` table:

```sql
CREATE TABLE changelog (
    id          INTEGER PRIMARY KEY,
    entity_type TEXT NOT NULL,        -- 'contact', 'deal', 'activity'
    entity_id   TEXT NOT NULL,        -- UUID of the affected record
    operation   TEXT NOT NULL,        -- 'insert', 'update', 'delete'
    field       TEXT,                 -- NULL for insert/delete, field name for update
    new_value   TEXT,                 -- JSON-encoded new value
    changed_at  INTEGER NOT NULL,     -- Unix timestamp (milliseconds)
    device_id   TEXT NOT NULL,        -- UUID assigned to this installation
    synced      INTEGER NOT NULL DEFAULT 0  -- 0 = pending, 1 = synced
);
```

SQLite triggers on each entity table automatically insert changelog records on INSERT, UPDATE, and DELETE.

### Push/Pull Pattern

```
Device A                    Sync Server                    Device B
   │                             │                             │
   │  GET /changelog?since=T     │                             │
   │────────────────────────────►│                             │
   │◄────────────────────────────│                             │
   │  [changelog entries since T]│                             │
   │                             │                             │
   │  POST /changelog             │                             │
   │  [local changes since last  │                             │
   │   successful push]          │                             │
   │────────────────────────────►│                             │
   │◄────────────────────────────│                             │
   │  200 OK                     │                             │
   │                             │  GET /changelog?since=T     │
   │                             │◄────────────────────────────│
   │                             │────────────────────────────►│
   │                             │  [merged changelog]         │
```

### Conflict Resolution

900CRM uses **last-write-wins** at the field level. For each field of each entity, the change with the most recent `changed_at` timestamp wins.

This is appropriate for CRM data because:
- Most CRM fields are independent (changing a phone number doesn't conflict with changing an address)
- Field-level LWW handles the common case of one device being offline for hours and then syncing
- The alternative (three-way merge or CRDTs) adds significant complexity for marginal benefit in this use case

For **delete** operations: a delete always wins over concurrent updates (tombstone semantics). Once a record is deleted, its ID is permanently recorded in a `tombstones` table to prevent resurrection during sync.

### Sync Server

The sync server is a simple HTTP API that stores and serves changelog entries. A reference implementation in Rust is provided in the `sync-server/` directory. The protocol is documented in `sync-server/API.md`. Teams can self-host it on any server with HTTP access.

The sync server does **not** need to be publicly accessible on the internet — it works on a local network (Wi-Fi in an office, hotspot in the field).

---

## Database Schema

The database is located at:
- **Windows:** `%APPDATA%\900CRM\data.db`
- **macOS:** `~/Library/Application Support/900CRM/data.db`
- **Linux:** `~/.local/share/900CRM/data.db`

### Schema Version

The schema version is stored in the `schema_version` table. The migration runner in `storage/schema.rs` applies migrations in sequence on startup.

```sql
CREATE TABLE schema_version (
    version     INTEGER PRIMARY KEY,
    applied_at  INTEGER NOT NULL
);
```

### Core Tables

```sql
-- Contacts (people and organizations)
CREATE TABLE contacts (
    id          TEXT PRIMARY KEY,           -- UUID v4
    name        TEXT NOT NULL,
    email       TEXT,
    phone       TEXT,
    company     TEXT,
    job_title   TEXT,
    website     TEXT,
    address     TEXT,
    notes       TEXT,
    created_at  INTEGER NOT NULL,           -- Unix ms
    updated_at  INTEGER NOT NULL
);

-- Tags associated with contacts
CREATE TABLE contact_tags (
    contact_id  TEXT NOT NULL REFERENCES contacts(id) ON DELETE CASCADE,
    tag         TEXT NOT NULL,
    PRIMARY KEY (contact_id, tag)
);

-- Pipeline stages (ordered list)
CREATE TABLE pipeline_stages (
    id          TEXT PRIMARY KEY,           -- UUID v4
    name        TEXT NOT NULL,
    position    INTEGER NOT NULL,           -- 0-indexed display order
    color       TEXT,                       -- hex color for kanban column
    is_won      INTEGER NOT NULL DEFAULT 0, -- treat as "closed won"
    is_lost     INTEGER NOT NULL DEFAULT 0  -- treat as "closed lost"
);

-- Deals
CREATE TABLE deals (
    id              TEXT PRIMARY KEY,       -- UUID v4
    name            TEXT NOT NULL,
    value           REAL,                   -- monetary value
    currency        TEXT DEFAULT 'USD',     -- ISO 4217
    stage_id        TEXT NOT NULL REFERENCES pipeline_stages(id),
    contact_id      TEXT REFERENCES contacts(id) ON DELETE SET NULL,
    expected_close  INTEGER,                -- Unix ms date
    notes           TEXT,
    created_at      INTEGER NOT NULL,
    updated_at      INTEGER NOT NULL
);

-- Activities
CREATE TABLE activities (
    id              TEXT PRIMARY KEY,       -- UUID v4
    type            TEXT NOT NULL,          -- 'task', 'call', 'meeting', 'email', 'note'
    title           TEXT NOT NULL,
    notes           TEXT,
    contact_id      TEXT REFERENCES contacts(id) ON DELETE SET NULL,
    deal_id         TEXT REFERENCES deals(id) ON DELETE SET NULL,
    due_at          INTEGER,                -- Unix ms, nullable
    completed_at    INTEGER,                -- NULL = not complete
    created_at      INTEGER NOT NULL,
    updated_at      INTEGER NOT NULL
);

-- Full-text search virtual table (FTS5)
CREATE VIRTUAL TABLE fts_index USING fts5(
    entity_type,    -- 'contact', 'deal', 'activity'
    entity_id,      -- UUID of the indexed entity
    content,        -- concatenated searchable text
    tokenize = 'unicode61 remove_diacritics 1'
);

-- Sync changelog (see Sync Protocol section)
CREATE TABLE changelog (
    id          INTEGER PRIMARY KEY,
    entity_type TEXT NOT NULL,
    entity_id   TEXT NOT NULL,
    operation   TEXT NOT NULL,
    field       TEXT,
    new_value   TEXT,
    changed_at  INTEGER NOT NULL,
    device_id   TEXT NOT NULL,
    synced      INTEGER NOT NULL DEFAULT 0
);

-- User settings (key-value)
CREATE TABLE settings (
    key     TEXT PRIMARY KEY,
    value   TEXT NOT NULL
);
```

### Indexes

```sql
CREATE INDEX idx_contacts_email ON contacts(email);
CREATE INDEX idx_contacts_updated ON contacts(updated_at DESC);
CREATE INDEX idx_deals_stage ON deals(stage_id);
CREATE INDEX idx_deals_contact ON deals(contact_id);
CREATE INDEX idx_activities_contact ON activities(contact_id);
CREATE INDEX idx_activities_deal ON activities(deal_id);
CREATE INDEX idx_activities_due ON activities(due_at);
CREATE INDEX idx_changelog_synced ON changelog(synced, changed_at);
```

---

## Extension Points

### Custom Fields (v1.1+)

Custom fields will be stored in a `custom_field_definitions` table and a `custom_field_values` EAV table:

```sql
CREATE TABLE custom_field_definitions (
    id          TEXT PRIMARY KEY,
    entity_type TEXT NOT NULL,  -- 'contact', 'deal', 'activity'
    name        TEXT NOT NULL,
    field_type  TEXT NOT NULL,  -- 'text', 'number', 'date', 'boolean', 'select'
    options     TEXT,           -- JSON array for 'select' type
    position    INTEGER NOT NULL
);

CREATE TABLE custom_field_values (
    entity_id    TEXT NOT NULL,
    field_id     TEXT NOT NULL REFERENCES custom_field_definitions(id),
    value        TEXT,
    PRIMARY KEY (entity_id, field_id)
);
```

The CRM engine and storage layer are designed with hooks for custom fields — they pass through an optional `custom_fields: HashMap<String, Value>` on all entity structs.

### Plugin SDK (v2.0+)

The plugin system is planned for v2.0. The plugin architecture follows the same two-layer model as the core:

- **Plugin backend** (Rust or WASM): registers additional Tauri commands via the plugin API
- **Plugin frontend** (JavaScript/TypeScript): adds UI panels, toolbar items, and menu entries via the Plugin Host API

Plugins are sandboxed: they can only call commands they explicitly declare in their `manifest.json`, and they cannot access the SQLite database directly — they must go through the plugin API.

See [plugins/README.md](plugins/README.md) for the current specification.

---

## MCP Readiness

MCP is a future optional package boundary, not a current runtime dependency. The desktop app and `crm-core` do not include a built-in AI agent, do not start an MCP server, do not bind a localhost listener, and do not require internet access, cloud services, or model providers.

The current codebase has external-client records, per-tool permission rows, proposed actions, audit logging, and Pending Actions/Audit Log UI surfaces that can support a future MCP implementation. The active permission modes are limited to `disabled`, `read_only`, and `draft_only`; broader schema-reserved modes are inactive. Approved proposed actions only record approval state and do not execute CRM mutations.

See [MCP Readiness Baseline](docs/MCP_READINESS.md) for the current status, non-goals, security gates, and future implementation acceptance checklist.

---

## Key Design Decisions

### Why Tauri Instead of Electron?

Electron bundles Chromium (~120 MB) into every installer. Tauri uses the system's built-in WebView, resulting in a ~3 MB binary. For users downloading over slow or metered connections — the primary audience for 900CRM — this difference is significant. Electron also uses 150–300 MB RAM at idle vs. 30–40 MB for Tauri. See [ADR-002](docs/adr/002-tauri-over-electron.md).

### Why Not a Web App or PWA?

PWAs have unreliable offline write capabilities, especially on iOS. Service Workers do not handle complex relational data operations (joins, aggregates, full-text search) well. A native desktop app gives full filesystem access for CSV import/export, works on older systems without browser configuration, and avoids the PWA fragmentation across browsers. See the research findings in [900crm-research.md](docs/research/900crm-research.md).

### Why SQLite Instead of a Managed Database?

SQLite is zero-configuration, embedded, and produces a single portable file. There is no server to run, no connection string to configure, no port to open. For a single-user desktop app in an offline context, it is the obvious choice. It handles millions of CRM records comfortably. See [ADR-001](docs/adr/001-sqlite-changelog-sync.md).

### Why Custom Changelog Sync Instead of CRDTs?

CRDTs (Conflict-free Replicated Data Types) are elegant for collaborative documents but add significant complexity for structured relational data. CRM records have clear ownership semantics (one user updates one record), making last-write-wins at the field level correct in practice. The custom changelog approach gives full control with no external dependencies. See [ADR-001](docs/adr/001-sqlite-changelog-sync.md).

### Why Svelte 5 Instead of React or Vue?

Svelte 5 with runes has the smallest runtime of any major frontend framework. It compiles to vanilla JavaScript with no virtual DOM overhead. This directly translates to faster startup and lower memory usage on constrained hardware. The 900 Labs team also uses Svelte in 900PDF, giving consistency across the ecosystem. See [ADR-003](docs/adr/003-svelte5-frontend.md).

### Why Rust for the Backend?

Rust provides memory safety without a garbage collector, native performance, and excellent SQLite bindings (`rusqlite`). It compiles to native code on all platforms, ensuring consistent performance regardless of OS. The Tauri ecosystem is Rust-native, making integration straightforward. See [ADR-002](docs/adr/002-tauri-over-electron.md).

### Why No Encryption by Default?

Adding encryption (e.g., SQLCipher) requires a key management strategy. For a single-user local app, the question becomes: where does the key live? Storing it on the same disk as the data provides limited security. Proper key management (OS keychain, user-provided passphrase) adds UX complexity that is a barrier for our target users. The recommendation is to use OS-level disk encryption (BitLocker, FileVault, LUKS), which is more effective and already available. Opt-in encryption may be added in a future release.

---

For questions about the architecture, open a [discussion](https://github.com/900-labs/900crm/discussions) or comment on the relevant issue. Architecture discussions are some of the most valuable contributions you can make.
