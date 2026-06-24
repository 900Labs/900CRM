# Sprint 020 - Deal Relationships UI Surface

Date: 2026-06-24
Branch: `codex/deal-relationships-ui-surface`
Scope: Frontend-only surface for Sprint 019 deal relationship fields in deal creation and pipeline cards.

## Summary

- Added organization and primary-contact selectors to the existing add-deal modal.
- Preserved stage prefill, contact-context prefill, currency normalization, and custom-field persistence in the modal.
- Displayed primary contact and organization labels on pipeline deal cards when the frontend has loaded matching contact and organization records.
- Added focused unit coverage for relationship label derivation and selector label fallbacks.

## Architecture Decisions

- The sprint uses the existing `createDeal` payload fields: `contactId` for the primary contact mirror and `organizationId` for the first-class organization link.
- No backend commands, schema changes, service changes, migrations, MCP behavior, AI behavior, sync behavior, or broad editor UI were added.
- Pipeline cards keep the backend `Deal` shape unchanged and derive names from already-loaded contacts and organizations in the frontend.
- Relationship label logic is isolated in a plain TypeScript utility so Svelte components stay compact and test coverage does not require a new component-test harness.
- Plain `.ts` files do not use Svelte runes.

## Validation

- [x] `npm run check`
- [x] `npm run test`
- [x] `npm run build`
- [x] `cargo fmt --all -- --check`
- [x] `CARGO_TARGET_DIR=/Volumes/T7/Code/Codex/900CRM-targets/deal-relationships-ui-surface cargo check --workspace`
- [x] `CARGO_TARGET_DIR=/Volumes/T7/Code/Codex/900CRM-targets/deal-relationships-ui-surface cargo test --workspace`
- [x] Raw SQL scan in `apps/desktop/src-tauri/src/commands`
- [x] Raw SQL scan in `crates/crm-core/src/crm_engine`
- [x] Plain `.ts` rune scan
- [x] Locale key parity check
- [x] `git diff --check origin/main...HEAD`
