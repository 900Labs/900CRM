# Security Policy

## Supported Versions

900CRM follows a **latest-stable** support policy. Security fixes are applied to the most recent release. We encourage all users to run the latest version.

| Version | Supported |
|---|---|
| Latest stable release | Yes — security fixes applied promptly |
| Previous minor version (x.N-1.x) | Best-effort for 6 months after new minor release |
| Previous major version | Best-effort for 6 months after new major release |
| Alpha / pre-release builds | No — not intended for production use |

---

## Reporting a Vulnerability

**Please do NOT report security vulnerabilities through GitHub Issues.** Issues are public, and disclosing a vulnerability before it is fixed could put users at risk.

### How to Report

Send an email to:

**security@900labs.com**

Use the subject line: `[900CRM Security] Brief description`

If possible, encrypt your message using our PGP key (available at [https://900labs.com/security.asc](https://900labs.com/security.asc)).

### What to Include

Please provide as much of the following as you can:

- A description of the vulnerability and its potential impact
- The component or file(s) affected (e.g., "CSV import in `src-tauri/src/commands/import_export.rs`")
- Steps to reproduce the issue
- A proof-of-concept or sample data (if applicable — remove any sensitive personal data)
- Your name or alias, if you would like to be credited in the security advisory (optional)

You do not need a complete exploit or a proposed fix. A clear description of the problem is enough to start.

### Response Timeline

| Timeframe | What happens |
|---|---|
| **Within 3 business days** | We acknowledge receipt of your report |
| **Within 10 business days** | We assess severity, reproduce the issue, and provide an initial response with our planned timeline |
| **Within 45 days** | For most vulnerabilities, we aim to release a patch and publish a security advisory |
| **Coordinated disclosure** | We work with you on the disclosure timeline. We ask that you do not publish details until a fix is released |

We follow a **coordinated disclosure** model. We will publish a public security advisory for all confirmed vulnerabilities once a fix is available.

### Credit

We will credit researchers who report valid security vulnerabilities in the security advisory, unless you prefer to remain anonymous. We are grateful for the work of security researchers.

---

## Security Design Principles

Understanding 900CRM's security model helps clarify what kinds of issues are in scope.

### No Network Calls

900CRM makes **zero outbound network requests** by default. There is no telemetry, no auto-update pings, no licence verification, no analytics, no cloud sync. The Tauri application is built with all network-related capabilities disabled in `tauri.conf.json`.

The only exception is the optional team sync feature (when explicitly configured by the user), which connects to a self-hosted sync server that the user controls. This connection is entirely opt-in and user-initiated.

If you observe unexpected network traffic from 900CRM that you did not initiate, please report it immediately — this would be a serious and unintended behaviour.

### No Telemetry

There is no analytics, no crash reporting, no usage tracking, and no data collection of any kind. Nothing about how you use the application — which contacts you manage, which deals you track, how often you use it — is recorded or transmitted anywhere.

### Local-Only Data Storage

All CRM data — contacts, deals, activities, settings — is stored in a SQLite database on the user's local machine:

- **Windows:** `%APPDATA%\900CRM\data.db`
- **macOS:** `~/Library/Application Support/900CRM/data.db`
- **Linux:** `~/.local/share/900CRM/data.db`

No data is ever written outside of the application data directory and files the user explicitly exports.

### Process Isolation

Tauri's two-process model provides a security boundary:

- The **WebView process** (frontend) runs in a sandboxed context and cannot access the file system, network, or OS directly.
- All file system and OS operations are performed by the **Rust backend** process, which validates all IPC commands strictly.
- Tauri v2's **capability system** enforces at build time which IPC commands the frontend is allowed to call.

### File System Access

900CRM's Rust backend only reads and writes:
- The application data directory (SQLite database, settings)
- Files that the user explicitly opens or saves via the system file picker
- Temporary files in the system temp directory during import/export operations (cleaned up immediately after use)

### CSV Import Security

The CSV import feature parses user-supplied files. If you discover that a specially crafted CSV file can cause 900CRM to crash, consume excessive memory, write data outside the intended scope, or behave unexpectedly, please report it. We treat CSV parsing as a potential attack surface.

### Data at Rest

The SQLite database is stored on the user's local filesystem. We do not apply additional encryption by default — this relies on the operating system's filesystem-level security. Users who need encrypted storage are encouraged to use OS-level disk encryption (BitLocker, FileVault, LUKS).

Future releases may include optional SQLCipher-based encryption for the database file. If you have input on this feature, please open a discussion.

---

## Out of Scope

The following are generally **not** in scope for security reports:

- Vulnerabilities in the operating system or WebView runtime (report these to the OS vendor or Microsoft/Apple/Google)
- Performance issues caused by very large datasets (open a regular issue instead)
- Issues that require the attacker to already have physical access to the machine and the user's OS account
- Social engineering attacks

If you are unsure whether something is in scope, err on the side of reporting it. We would rather review something out of scope than miss a real vulnerability.

---

Thank you for helping keep 900CRM secure.
