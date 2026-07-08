# Product Review and Competitive Benchmark

Date: 2026-07-08
Branch: `codex/reports-hub`

This document is a product-depth review of 900CRM's current menu surface and a
benchmark against comparable CRM products. It is intentionally direct: 900CRM
has a strong offline/local-first foundation, but the visible product still feels
basic because several high-frequency CRM workflows are either hidden, shallow,
or not implemented.

## Executive Summary

900CRM is differentiated by local-first desktop operation, no account
requirement, local SQLite data ownership, backup/restore, import/export depth,
audit evidence, and a narrow optional MCP stdio boundary. That is meaningful.

The weakness is the daily CRM experience. As of Sprint 108, the left navigation
is visually grouped into:

- Workspace: Dashboard, Contacts, Organizations, Pipeline, Activities, Reports
- Review: Pending Actions, Audit Log
- Admin: Settings

That grouping is a better information architecture than the original flat menu,
the dashboard now has a first-run starter checklist plus optional synthetic
sample workspace, and contact detail now starts to behave like a Customer 360
workspace with open-deal, pipeline, next-follow-up, recent-activity, and
attention-status summary signals. Organization/account detail now has its own
workspace, and contact/account timelines now show relationship breadcrumbs for
linked contacts, accounts, and deals. Pipeline and Activities now have richer
daily-work surfaces, and Reports has a dedicated route for current pipeline and
activity health. The product still undersells its
foundation because several core CRM jobs remain shallow or absent. Comparable
CRMs make sales work feel richer through leads, inbox and communication
history, calendar views, reports, saved views, automation, guided onboarding,
and clearer deal workspaces.

The next phase should therefore be a product-depth phase, not another foundation
phase. The priority is to make 900CRM feel useful in the first 10 minutes for a
real small business owner.

## Current Menu Review

| Area | Current state | Product issue |
|---|---|---|
| Dashboard | KPI cards, activity feed, quick actions, first-run starter checklist, optional synthetic sample workspace | Useful start, but not yet a true command center. No saved goals, no "what needs attention" queue, and no personalized daily priority logic. |
| Contacts | Search, type filter, duplicate review, import/export, custom-field filtering, contact detail Customer 360 summary, relationship-aware activity timeline | Contacts and lead/account work are blended. The detail view now has useful at-a-glance and timeline context, but there is no lead capture stage, saved segment, list view preset, files/links section, or deal-detail context. |
| Organizations | List, create/edit, notes/tags, contact linking, account detail workspace with linked people/deals/activity and relationship-aware account timeline | Account detail is now useful, but list-level account management still lacks owner, health, next step, and saved account views. |
| Pipeline | Kanban by stage, custom-field filter, deal cards, deal guidance drawer with weighted forecast, stale/overdue/follow-up status, linked activities, and a board-level forecast/stage-health overview | Stronger daily sales surface. Still needs true historical stage conversion summaries, deeper deal editing/detail routing, and stronger drag/drop confidence cues. |
| Activities | Task/call/meeting/email follow-up workbench with due buckets, summary counts, quick snooze/reschedule, completion, filters, and relationship breadcrumbs | Much stronger daily work surface. Still needs a true calendar grid, reminders, recurrence, saved activity views, and optional external calendar sync. |
| Reports | Dedicated Workspace route for current pipeline and activity health, current-stage funnel ratios, due buckets, and activity type mix | Usable alpha reporting. Still needs saved filters, stale deal reports, source/owner dimensions, exportable snapshots, and true historical conversion after stage history exists. |
| Audit Log | Read-only table grouped under Review | Good trust feature, now visually separated from daily work. Later, it may still move deeper into Admin once Review counts are more prominent. |
| Pending Actions | Review queue for proposed actions grouped under Review | Important for MCP/external-client safety. The next improvement is to show high-priority pending counts without making this a daily-work screen. |
| Settings | Locale/theme/date/currency, backup/restore, import/export, email, integrations, section jump bar | Still dense, but now easier to scan. Later work should split Data Management, Integrations, Appearance, and Admin/Safety into clearer settings sections or pages. |

## What Comparable CRMs Emphasize

The benchmark is based on official product pages and feature documentation where
available.

| Product | What it emphasizes | Relevance for 900CRM |
|---|---|---|
| [HubSpot CRM](https://www.hubspot.com/products/crm) | Unified customer data, contact management, CRM import, deals, tasks/activities, pipeline management, reporting dashboards, integrations, mobile, AI assistant, chatbots, shared inbox, email tracking/templates. | 900CRM has the local data core but lacks the "everything in one customer workspace" feeling: inbox/history, saved lists, onboarding, and richer dashboards. |
| [Pipedrive](https://www.pipedrive.com/en/features/pipeline-management) | Pipeline-first selling, customizable stages and fields, activity tracking, automations, pipeline metrics, hundreds of integrations, LeadBooster, email and document add-ons. | 900CRM's pipeline should become the strongest workflow: drag/drop confidence, next-step prompts, stale deal warnings, conversion/funnel metrics, forecast view. |
| [Zoho CRM](https://www.zoho.com/crm/features.html) | AI assistance, sales force automation, lead management, forecasting, pipeline, territory management, workflows, cadences, process automation, page layouts, journey orchestration, customization. | 900CRM cannot match cloud AI/automation for alpha, but it needs lightweight local automation and lead lifecycle support to avoid feeling static. |
| [Odoo CRM](https://www.odoo.com/app/crm) | Opportunity Kanban, scheduled follow-ups, email/live chat/SMS/VoIP integrations, quotations, forecasts, dashboards, automation, AI lead scoring, integrated business apps. | 900CRM should not become ERP-heavy, but quotes, follow-up scripts, forecast dashboards, and account/deal summaries are high-value small-business workflows. |
| [SuiteCRM](https://suitecrm.com/what-is-suitecrm/) | Open-source control, sales, marketing, reports, dashboards, customer 360, workflow, activity management, case management, configuration studio. | SuiteCRM is broader but server-heavy. 900CRM can win on simplicity/offline, while borrowing customer 360, reports, workflow, and admin customization concepts. |
| [EspoCRM](https://www.espocrm.com/features/) | Leads, opportunities, accounts, contacts, calendar, email sync/sending/templates/mass email, stream, cases/portal/knowledge base, documents, campaigns, entity manager, roles, import/export, reports, workflows, VoIP, projects, meeting scheduling. | EspoCRM shows how wide even lean open-source CRMs get. 900CRM's alpha should pick a few daily workflows, not chase every module. |
| [Twenty](https://twenty.com/) | Modern open-source CRM building blocks, custom data model/layout/automation, familiar Notion-like interface, command menu, real-time data, AI chat, opportunities and company views. | Twenty raises the UX bar. 900CRM should add command/search ergonomics, modern record layouts, saved views, and faster navigation before adding many deep modules. |

## Benchmark Scorecard

Scale: 0 = absent, 1 = foundation only, 2 = usable alpha, 3 = competitive.

| Capability | 900CRM today | Target for credible alpha |
|---|---:|---:|
| Local/offline ownership | 3 | 3 |
| Contacts and organizations | 2 | 3 |
| Pipeline/deals | 2 | 3 |
| Activities/tasks | 2 | 3 |
| Dashboard/reporting | 2 | 2 |
| Search/navigation speed | 2 | 3 |
| Import/export/backup | 3 | 3 |
| Lead lifecycle | 0 | 2 |
| Account/customer 360 view | 2 | 2 |
| Calendar/follow-up workflow | 1 | 2 |
| Email/communication history | 1 | 2 |
| Automation/workflows | 1 | 2 |
| Forecasting | 1 | 2 |
| Onboarding/sample data | 2 | 3 |
| Release/installability | 1 | 2 |
| Mobile/team/cloud collaboration | 0 | Deferred |

## Highest-Impact Gaps

### 1. First-run value is started, not complete

A new user needs to know what to do immediately. Sprint 101 added a dashboard
starter checklist and optional local synthetic sample workspace. That closes the
blank-dashboard problem, but competitors still go further with starter
pipelines, product tours, guided imports, and richer empty states across every
primary module.

Recommended outcome: extend first-run guidance into module empty states,
import/backup prompts, and customer/deal workspace guidance without adding
marketing-style tours.

### 2. Customer workspaces now cover contacts and accounts, but still need attached context

The contact detail page exists, notes/tags/custom fields exist, and Sprint 102
added a contact-level Customer 360 summary for open deals, open pipeline value,
next follow-up, recent activity, and attention status. Sprint 103 added a
first-class organization/account detail workspace with profile metadata, linked
people, linked deals, account activity, account health, direct account routes,
and global-search routing into account records. That makes core records feel
less like static tables. Sprint 104 deepened the activity timeline by including
relationship-link-only contact activity and showing contact, organization, and
deal breadcrumbs in contact and account detail timelines. The product still
needs the fuller "customer 360" pattern: files/links, deal detail context, and
next-action guidance across more workflows.

Recommended outcome: richer contact and organization detail workspaces before
adding unrelated modules.

### 3. Pipeline guidance and current-stage metrics are started, but deal workflows are not deep enough yet

The Kanban board exists, and Sprint 105 added a deal guidance drawer with
weighted forecast, next activity context, stale/overdue/follow-up states, and
an Add Follow-Up path. A salesperson can now inspect a deal without leaving the
board. Sprint 106 added a board-level forecast and stage-health overview that
groups open value and weighted forecast by currency, summarizes close-date
health, shows current Closed Won/Lost win-rate context, and distributes
guidance risk by stage. This is current-stage operational guidance, not true
historical stage-conversion analytics.

Recommended outcome: deeper deal editing/detail routing, stronger drag/drop
confidence cues, and true stage-conversion summaries only after the data model
stores stage-transition history.

### 4. Activities now have a follow-up workbench, but not a full calendar

Sprint 107 moved Activities from a flat list toward daily follow-up discipline:
the route now groups work into Overdue, Today, This Week, Later,
Unscheduled, and Completed buckets, shows priority counts, and allows quick
snooze/reschedule/complete actions while preserving relationship context.
This closes the first daily-work gap, but users who plan heavily by time still
need a true day/week calendar grid, reminders, recurrence, saved views, and
optional external calendar sync.

Recommended outcome: add a true calendar/planning layer only after the
follow-up workbench settles.

### 5. Reports now have a home, but they are still operational snapshots

Sprint 108 moved the embedded Dashboard report cards into a dedicated Reports
route under Workspace navigation. The page shows current pipeline win rate,
open/closed counts, current stage distribution, current-stage funnel ratios,
activity completion, overdue rate, due buckets, and type breakdown. The labels
make clear that funnel ratios are current-stage comparisons, not historical
stage-transition analytics.

Recommended outcome: keep Reports focused on current operational health until
the data model stores stage-transition history and richer report dimensions
such as owner/source are available. Then add saved filters, exportable snapshots,
and historical conversion reporting.

### 6. Admin/safety items crowd primary navigation

Audit Log and Pending Actions are important, but they are not daily top-level
CRM work for most users. They make the app feel technical.

Recommended outcome: group these under a Review/Admin section, or show them
only when counts require attention.

### 7. Settings is overloaded

Settings contains appearance, locale, date/currency, backup, import/export,
email, and external clients. This should be a settings shell with sections, or
separate Data Management and Integrations pages.

Recommended outcome: split Settings into Appearance, Data Management,
Integrations, Email, and Admin/Safety sections.

## Recommended Product-Depth Sprints

These are ordered to improve perceived product value fastest.

1. **Docs and Navigation Reframe** - completed for the current visible grouping
   - Add a durable docs index.
   - Keep sprint logs as audit evidence, not the main entry point.
   - Reframe current alpha status as "source-ready, product-depth in progress,
     release packaging pending."
   - Group admin/safety nav items visually or document the intended grouping.

2. **First-Run Onboarding and Sample Data** - completed for the dashboard
   starter layer
   - Added a first-run checklist and optional synthetic sample CRM dataset.
   - Guides users through first contact or organization, first deal, and first
     follow-up from the dashboard.
   - Remaining work: module-specific empty states, guided import/backup prompts,
     and richer onboarding inside customer/deal workspaces.

3. **Customer 360 Workspaces** - contact/account summaries and timeline
   breadcrumbs completed
   - Added contact detail open-deal, open-pipeline, next-follow-up,
     recent-activity, and attention-status summary.
   - Added organization/account detail with account profile, linked people,
     linked deals, account activity, account health, direct account routes, and
     global-search routing into contact/account record views.
   - Added relationship-aware activity timelines for contact and account detail,
     including link-only contact activities and contact/account/deal breadcrumbs.
   - Remaining work: files/links, deal detail context, and richer record routing
     for records that do not yet have detail pages.

4. **Pipeline Depth** - deal guidance and current-stage metrics completed
   - Added deal drawer from Pipeline cards with existing stage, value,
     probability, weighted forecast, expected close, description, created/updated
     dates, linked activities, and Add Follow-Up action.
   - Added guidance badges for Needs Follow-Up, Overdue, Stale, On Track, Closed
     Won, and Closed Lost.
   - Added board-level open pipeline value, weighted forecast, next-30-day close
     forecast, close-date health, win-rate context, and per-stage current health
     metrics derived from the current filtered board.
   - Remaining work: deeper deal editing/detail routing, stronger drag/drop
     confidence cues, and true conversion summaries after stage history exists.

5. **Activities Calendar and Follow-up Center** - follow-up workbench completed
   - Added due buckets for Overdue, Today, This Week, Later, Unscheduled, and
     Completed.
   - Added quick reschedule, snooze, complete/incomplete, and preserved
     relationship breadcrumbs.
   - Remaining work: true day/week calendar grid, reminders, recurrence, saved
     activity views, and optional external calendar sync.

6. **Reports Hub** - first dedicated route completed
   - Moved dashboard reports into a Reports page under Workspace navigation.
   - Added current pipeline win rate, open/closed counts, current stage
     distribution, current-stage funnel ratios, activity completion, overdue
     rate, due buckets, and activity type mix.
   - Remaining work: saved filters, stale deal reporting, source/owner
     breakdown where supported, exportable report snapshots, and true
     historical conversion only after stage history exists.

7. **Local Automation Lite** - explicit local suggestions completed
   - Added a Pipeline stage-move prompt that drafts a follow-up only after a
     user moves an open deal with no linked next activity. Nothing is saved
     until the user reviews and saves the Add Activity modal.
   - Added a Dashboard attention strip for overdue and due-today follow-ups
     using the frontend local-day bucketing rules from the Activities
     workbench.
   - Added import wizard guidance that clarifies source/tag columns are not
     applied automatically to contacts and that tag links require local IDs.
   - Remaining work: saved automation rules, recurring reminders, full
     workflow automation, owner/source dimensions, and auto-tagging only after
     import/rollback semantics explicitly support it.

8. **Alpha Packaging and Smoke Evidence**
   - Local macOS smoke evidence is now repeatable through
     `npm run release:macos:smoke:local`, which builds/verifies a local
     headless DMG, generates macOS-only metadata/checksums/SBOM, verifies the
     local artifact tree, and performs a mounted-DMG layout smoke.
   - Remaining work: resolve the GitHub Actions billing/spending-limit blocker,
     produce Actions-backed Windows, macOS, and Linux artifacts, verify
     downloaded workflow outputs, and smoke test real installers on each target
     platform.

## Product Principles Going Forward

- Prefer fewer deeper workflows over many shallow modules.
- Keep the offline/local-first promise as the differentiator.
- Avoid copying enterprise CRM complexity unless it directly helps a small
  business follow up, sell, or retain customers.
- Treat release/installability as a separate track from product depth.
- Keep MCP, sync server, and built-in AI deferred unless they improve a visible
  user workflow and pass explicit security gates.

## Acceptance Criteria for the Next Review

The next product review should test these concrete questions:

- Can a new user understand what to do in the first 10 minutes?
- Can a user see every important fact about a customer without opening several
  unrelated screens?
- Can a user identify which deal or customer needs attention today?
- Can a user recover from import mistakes and trust local backup/restore?
- Can the app be installed by a non-technical alpha user from a real artifact?
- Does the left navigation reflect daily CRM work rather than implementation
  internals?
