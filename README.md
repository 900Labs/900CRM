```
  ___   ___   ___  _____  ____  __  __
 / _ \ / _ \ / _ \|  ___||  _ \|  \/  |
| (_) | | | | | | | |_   | |_) | |\/| |
 \__, | |_| | |_| |  _|  |  _ <| |  | |
   /_/ \___/ \___/|_|    |_| \_\_|  |_|

  Offline-first CRM for developing nations.
```

[![License: Apache 2.0](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)
[![Build Status](https://img.shields.io/github/actions/workflow/status/900-labs/900crm/ci.yml?branch=main)](https://github.com/900-labs/900crm/actions)
[![Platform: Windows | macOS | Linux](https://img.shields.io/badge/Platform-Windows%20%7C%20macOS%20%7C%20Linux-lightgrey.svg)](https://github.com/900-labs/900crm/releases)
[![Languages: 8](https://img.shields.io/badge/Languages-8-brightgreen.svg)](https://github.com/900-labs/900crm#language-support)
[![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg)](CONTRIBUTING.md)
[![Contributor Covenant](https://img.shields.io/badge/Contributor%20Covenant-2.1-4baaaa.svg)](CODE_OF_CONDUCT.md)

---

## The 900 Mission

> 900 Labs is building enterprise-grade open source tools for the **900 million+ people** in developing economies who are priced out of the software that modern businesses depend on.

A small business in Lagos pays the same $50/seat/month for CRM software as a Fortune 500 company in New York — but operates in an economy where the average monthly salary is $150. A sales team in Nairobi needs the same pipeline management tools as one in London, but cannot afford $25/user/month for software that breaks the moment the internet drops.

AI has made it possible to build production-quality software at a fraction of the traditional cost. We believe that advantage should flow to the people who need it most — not just to those who can already afford it.

Every tool we release is **free and open source** under permissive licenses. No freemium traps, no usage limits, no surprise paywalls. **If we build it, you own it.**

Learn more at [900labs.com/impact](https://www.900labs.com/impact).

---

## What is 900CRM?

900CRM is the second tool from [900 Labs](https://www.900labs.com) — a free, open-source desktop CRM (Customer Relationship Management) application designed to work entirely offline. No internet connection required, ever. Built with [Tauri v2](https://tauri.app/) (Rust) and [Svelte 5](https://svelte.dev/), it runs on Windows, macOS, and Linux with a small installer footprint and low hardware requirements.

900CRM is built especially for:
- **Small businesses and entrepreneurs** in regions with unreliable connectivity
- **Sales teams and field representatives** who work offline or in areas with metered data
- **NGOs and non-profits** that need contact management without cloud subscriptions
- **Schools, clinics, and government offices** that require local data sovereignty
- **Anyone** who wants a capable CRM that doesn't cost anything and doesn't phone home

Your business data stays on your device. Nothing is ever uploaded anywhere.

<!-- screenshot -->

---

## Features

### Contacts
Manage your full network of people and organizations. Add custom fields, tag contacts, write notes, and attach file links. Search across your entire contact database instantly.

<!-- screenshot -->

### Pipeline
Visual kanban-style deal pipeline with drag-and-drop stages. Track every deal from first contact to close. See the total value of your pipeline at a glance. Customize stages to match your sales process.

<!-- screenshot -->

### Activities
Link tasks, calls, meetings, and follow-ups to any contact or deal. Set due dates and receive desktop notifications. Never miss a follow-up again. Activity history gives you a complete timeline of every interaction.

<!-- screenshot -->

### Dashboard
At-a-glance business metrics on every launch: pipeline value, deals by stage, upcoming tasks, recently modified contacts, and activity completion rates. No configuration needed — it works out of the box.

<!-- screenshot -->

### Search
Full-text search across contacts, deals, and activities. Instant results as you type. Filter by entity type, date range, or tag. Works 100% offline — every search query stays on your machine.

### Import / Export
Bring your existing data in with one-click CSV import for contacts, deals, and activities. Export any data set to CSV for use in spreadsheets, accounting tools, or data migration. Your data is always portable.

### Internationalization (i18n)
The entire interface is localized. Switch languages in settings instantly. Full right-to-left (RTL) layout support for Arabic and future RTL languages. Community translations welcome.

### Offline-First by Design
Every single feature works without an internet connection. There is no "offline mode" to enable — offline is the default. Optional sync to a local-network server is available for teams sharing data across devices.

---

## Why 900CRM?

**The cost barrier is real.**
SaaS CRM pricing ranges from $14 to $500 per user per month. In Sub-Saharan Africa, software is 10–13% more expensive than in the US in purchasing power terms. An ERP system in Kenya can cost three times the price of a basic computer. For millions of small businesses, CRM software is simply not affordable.

**Connectivity cannot be assumed.**
Fixed broadband in Sub-Saharan Africa costs over 20% of per capita income. Mobile data is expensive and unreliable in many regions. Every existing open-source CRM — SuiteCRM, EspoCRM, Odoo, Vtiger — requires a server and constant internet access. None of them work offline-first on the desktop.

**Existing tools leave a gap nobody fills.**

| Tool | Critical Gap |
|---|---|
| Salesforce, HubSpot | $25–$500/user/month, requires internet, cloud-only |
| SuiteCRM | Requires server + technical setup, no offline mode, dated UI |
| EspoCRM | Still requires server, no offline mode |
| Odoo | Free version very limited, complex setup, heavy resources |
| Vtiger | Best features cloud-only, self-hosting requires expertise |

900CRM fills the gap: a **capable, beautiful CRM that works on a $200 laptop with no internet connection and no server** — and costs nothing to use.

**Your data belongs to you.**
There is no account to create, no cloud to sync to, no telemetry, no analytics. Your contacts, deals, and business relationships live in a SQLite database on your machine. You can back it up with a USB drive, move it to a new computer, or inspect it with any SQLite tool.

---

## Tech Stack

| Technology | Role | Why we chose it |
|---|---|---|
| [Tauri v2](https://tauri.app/) | Desktop shell & native OS integration | Rust-based, ~3 MB binaries (vs. 150+ MB for Electron), no bundled browser engine |
| [Rust](https://www.rust-lang.org/) | Backend logic, data engine, IPC | Memory-safe, fast, compiles to native code on all platforms |
| [Svelte 5](https://svelte.dev/) | Frontend UI | Smallest runtime of any major framework, runes-based reactivity, no virtual DOM |
| [SQLite](https://www.sqlite.org/) | Local data storage | Zero-configuration, single-file database, handles millions of CRM records |
| [TypeScript](https://www.typescriptlang.org/) | Frontend type safety | Catches bugs at compile time, improves IDE support |

**Why not a web app or PWA?**
PWAs have limited offline write capabilities, especially on iOS. Service Worker caching does not handle complex relational data operations well. A native desktop app gives full filesystem access for data export/import, works on older systems without browser configuration, and is consistent with 900 Labs' philosophy: local-first, no cloud dependency.

---

## Quick Start

This section walks you through getting 900CRM running locally for development. To **use** 900CRM, download the installer from the [Releases page](https://github.com/900-labs/900crm/releases).

### Prerequisites

- **Node.js 20 or later** — [nodejs.org](https://nodejs.org/)
  - Verify: `node --version` → `v20.x.x` or higher
- **Rust stable 1.70 or later** — [rustup.rs](https://rustup.rs/)
  - Verify: `rustc --version` and `cargo --version`
- **Git** — [git-scm.com](https://git-scm.com/)

**Platform-specific dependencies:**

#### Linux (Ubuntu / Debian)

```bash
sudo apt update
sudo apt install -y \
  libwebkit2gtk-4.1-dev \
  build-essential \
  curl \
  wget \
  file \
  libssl-dev \
  libayatana-appindicator3-dev \
  librsvg2-dev \
  libgtk-3-dev
```

For other Linux distributions, see [Tauri prerequisites](https://tauri.app/start/prerequisites/).

#### Windows

1. Install [Microsoft C++ Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) with the "Desktop development with C++" workload.
2. WebView2 is included with Windows 10 (1803+) and Windows 11. Older systems: install the [WebView2 Runtime](https://developer.microsoft.com/microsoft-edge/webview2/).

#### macOS

```bash
xcode-select --install
```

That is all. Rust handles the rest.

### Step-by-Step Setup

**Step 1 — Clone the repository**

```bash
git clone https://github.com/900-labs/900crm.git
cd 900crm
```

**Step 2 — Install Node.js dependencies**

```bash
npm install
```

**Step 3 — Install the Tauri CLI**

```bash
cargo install tauri-cli --version "^2.0"
```

This compiles the Tauri CLI from source. The first run takes several minutes; subsequent runs use the cached build.

**Step 4 — Run in development mode**

```bash
cargo tauri dev
```

This starts the Svelte dev server with hot-reload, compiles the Rust backend, and opens the 900CRM window. Svelte changes hot-reload instantly; Rust changes trigger a recompile (10–30 seconds).

---

## Download

Pre-built installers are available on the [Releases page](https://github.com/900-labs/900crm/releases).

| Platform | Installer | Minimum OS |
|---|---|---|
| **Windows** | `.msi` installer | Windows 10 (1803+) |
| **macOS** | `.dmg` disk image | macOS 11 Big Sur (Intel & Apple Silicon) |
| **Linux** | `.deb` package | Ubuntu 20.04 / Debian 11 |
| **Linux** | `.AppImage` | Any modern Linux distribution |

The installer is self-contained and does not require an internet connection to install. You can distribute it on a USB drive.

---

## Language Support

900CRM is fully internationalized. Translations live in `src/lib/i18n/` and can be contributed by anyone.

| Language | Code | Coverage | Status |
|---|---|---|---|
| English | `en` | 100% | Complete — base language |
| French | `fr` | 100% | Complete |
| Spanish | `es` | 100% | Complete |
| Arabic | `ar` | 100% | Complete (RTL layout supported) |
| Swahili | `sw` | 100% | Complete |
| Portuguese (Brazil) | `pt` | 100% | Complete |
| Vietnamese | `vi` | 100% | Complete |
| Hindi | `hi` | 85% | In progress — contributions welcome |

To add a new language or improve an existing translation, see the [Translation Guide in CONTRIBUTING.md](CONTRIBUTING.md#translation-guide).

---

## Architecture Overview

900CRM uses a two-process model provided by Tauri:

```
┌────────────────────────────────────────────────────────────┐
│                      Operating System                       │
│                                                            │
│  ┌─────────────────────────┐  ┌────────────────────────┐  │
│  │   Rust Backend           │  │  WebView (UI)           │  │
│  │   (src-tauri/src/)       │◄─┤  (Svelte 5 / TS)       │  │
│  │                         │  │                        │  │
│  │  • CRM Engine            │  │  • Dashboard           │  │
│  │  • Storage (SQLite)      │  │  • Contacts view       │  │
│  │  • Sync engine           │  │  • Pipeline kanban     │  │
│  │  • IPC command server    │  │  • Activities feed     │  │
│  │  • Import/Export         │  │  • Search              │  │
│  └─────────────────────────┘  └────────────────────────┘  │
│            ▲                            │                   │
│            └──── Tauri IPC bridge ──────┘                   │
└────────────────────────────────────────────────────────────┘
                            │
                     SQLite database
                  (~/.local/share/900CRM/)
```

For a deep dive into the module structure, data flow, sync protocol, and design decisions, see [ARCHITECTURE.md](ARCHITECTURE.md).

---

## Project Structure

```
900crm/
├── src/                          # Svelte 5 frontend source
│   ├── lib/
│   │   ├── components/           # Reusable Svelte components
│   │   │   ├── contacts/         # Contact list, detail, editor
│   │   │   ├── pipeline/         # Kanban board, deal cards
│   │   │   ├── activities/       # Activity feed, task editor
│   │   │   ├── dashboard/        # Metric cards, charts
│   │   │   └── shared/           # Buttons, inputs, modals
│   │   ├── stores/               # Svelte 5 rune-based state
│   │   ├── api/                  # Tauri invoke wrappers
│   │   ├── i18n/                 # Translation JSON files
│   │   │   ├── en.json           # English (base)
│   │   │   ├── fr.json           # French
│   │   │   ├── es.json           # Spanish
│   │   │   ├── ar.json           # Arabic (RTL)
│   │   │   ├── sw.json           # Swahili
│   │   │   ├── pt.json           # Portuguese (Brazil)
│   │   │   ├── vi.json           # Vietnamese
│   │   │   └── hi.json           # Hindi
│   │   └── utils/                # Shared helpers
│   └── routes/                   # SvelteKit routes
│       ├── +layout.svelte        # App shell, nav, sidebar
│       ├── dashboard/            # Dashboard route
│       ├── contacts/             # Contacts routes
│       ├── pipeline/             # Pipeline / deals route
│       ├── activities/           # Activities route
│       └── settings/             # Settings route
├── src-tauri/                    # Tauri / Rust backend
│   ├── src/
│   │   ├── main.rs               # Entry point, Tauri setup
│   │   ├── commands/             # Tauri IPC command handlers
│   │   │   ├── contacts.rs       # CRUD for contacts
│   │   │   ├── deals.rs          # CRUD for deals + pipeline
│   │   │   ├── activities.rs     # CRUD for activities
│   │   │   ├── search.rs         # Full-text search
│   │   │   ├── import_export.rs  # CSV import/export
│   │   │   └── settings.rs       # User preferences
│   │   ├── crm_engine/           # Business logic layer
│   │   │   ├── contacts.rs       # Contact domain logic
│   │   │   ├── deals.rs          # Deal / pipeline logic
│   │   │   ├── activities.rs     # Activity logic
│   │   │   └── search.rs         # Search indexing
│   │   ├── storage/              # Database access layer
│   │   │   ├── schema.rs         # SQLite schema + migrations
│   │   │   ├── queries.rs        # Typed query functions
│   │   │   └── sync.rs           # Changelog-based sync
│   │   └── utils/                # Shared utilities
│   ├── capabilities/             # Tauri v2 permission definitions
│   ├── icons/                    # App icons (all sizes)
│   └── tauri.conf.json           # Tauri build configuration
├── plugins/                      # Community plugin directory
│   └── README.md                 # Plugin development guide
├── tests/                        # Integration tests
├── .github/
│   ├── workflows/
│   │   ├── ci.yml                # CI pipeline
│   │   └── release.yml           # Automated release builds
│   └── ISSUE_TEMPLATE/
│       ├── bug_report.md
│       └── feature_request.md
├── ARCHITECTURE.md               # Technical architecture guide
├── CHANGELOG.md
├── CODE_OF_CONDUCT.md
├── CONTRIBUTING.md
├── LICENSE
├── README.md                     # You are here
├── SECURITY.md
├── package.json
├── svelte.config.js
├── tsconfig.json
└── vite.config.ts
```

---

## Contributing

We welcome contributions from everyone — whether you are fixing a typo, translating the app into your language, writing a test, or building a new feature.

Please read [CONTRIBUTING.md](CONTRIBUTING.md) for full details on how to get started. If you have never contributed to an open-source project before, that document is written especially for you.

**Good First Issues:** Look for issues tagged [`good first issue`](https://github.com/900-labs/900crm/issues?q=is%3Aissue+is%3Aopen+label%3A%22good+first+issue%22) on the issue tracker — well-scoped tasks that don't require deep knowledge of the codebase.

---

## Roadmap

### v1.0.0 — Initial Release (March 2026)
- [x] Contacts management with custom fields and tags
- [x] Visual kanban pipeline with drag-and-drop
- [x] Activities: tasks, calls, meetings
- [x] Dashboard with at-a-glance metrics
- [x] Full-text search across all entities
- [x] CSV import and export
- [x] 6 languages (EN, FR, ES, AR, SW, HI)
- [x] Windows, macOS, Linux installers

### v1.1.0 — Planned
- [x] Custom fields on any entity type
- [x] Reports and conversion funnels
- [x] Desktop reminders and notifications
- [ ] Email integration (IMAP/SMTP, optional)
- [x] Multi-currency display
- [x] Additional languages: Portuguese (Brazilian), Vietnamese
- [ ] Additional languages: Hausa, Bengali

### v2.0.0 — Planned (Long-term)
- [ ] Plugin system for community extensions
- [ ] Optional multi-device sync (self-hosted server)
- [ ] Mobile companion app (Tauri mobile)
- [ ] Quotes and invoice generation (PDF output)
- [ ] WhatsApp integration for business communication

---

## Community

- **Discussions:** [github.com/900-labs/900crm/discussions](https://github.com/900-labs/900crm/discussions) — ask questions, share ideas, show what you have built
- **Issue Tracker:** [github.com/900-labs/900crm/issues](https://github.com/900-labs/900crm/issues) — report bugs, request features
- **Security:** See [SECURITY.md](SECURITY.md) for how to report vulnerabilities privately

---

## 900 Labs Ecosystem

900CRM is part of the growing 900 Labs open infrastructure suite:

| Tool | Description | Status |
|---|---|---|
| [900PDF](https://github.com/900-labs/900pdf) | Offline-first PDF viewer and editor | Released |
| **900CRM** | Offline-first CRM for developing nations | Released |
| More coming | Accounting, invoicing, inventory — and more | Planned |

Every tool in the 900 Labs suite shares the same principles: offline-first, local data, permissive license, no cloud dependency.

Visit [900labs.com](https://www.900labs.com) to learn more.

---

## License

900CRM is licensed under the **Apache License, Version 2.0**.

This means you are free to use, modify, and distribute 900CRM for any purpose — including commercially — as long as you include the license notice. The Apache 2.0 license also includes an explicit patent grant from all contributors.

See the full [LICENSE](LICENSE) file for details.

---

## Acknowledgments

900CRM stands on the shoulders of outstanding open-source projects:

- **[Tauri](https://tauri.app/)** — the framework that makes small, fast, secure desktop apps possible
- **[Svelte](https://svelte.dev/)** — a frontend framework that respects the user's hardware
- **[SQLite](https://www.sqlite.org/)** — the world's most deployed database, public domain, the right choice for local-first software
- **[Rust](https://www.rust-lang.org/)** — a language that makes systems programming safe and accessible

Most importantly, this project is dedicated to the entrepreneurs, sales teams, NGO workers, and small business owners in communities with limited connectivity who deserve the same quality tools as anyone else. You are not an afterthought — you are the reason this exists.
