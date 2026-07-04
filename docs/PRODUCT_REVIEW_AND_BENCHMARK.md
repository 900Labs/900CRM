# Product Review and Competitive Benchmark

Date: 2026-07-02
Branch: `codex/docs-product-benchmark-review`

This document is a product-depth review of 900CRM's current menu surface and a
benchmark against comparable CRM products. It is intentionally direct: 900CRM
has a strong offline/local-first foundation, but the visible product still feels
basic because several high-frequency CRM workflows are either hidden, shallow,
or not implemented.

## Executive Summary

900CRM is differentiated by local-first desktop operation, no account
requirement, local SQLite data ownership, backup/restore, import/export depth,
audit evidence, and a narrow optional MCP stdio boundary. That is meaningful.

The weakness is the daily CRM experience. As of Sprint 102, the left navigation
is visually grouped into:

- Workspace: Dashboard, Contacts, Organizations, Pipeline, Activities
- Review: Pending Actions, Audit Log
- Admin: Settings

That grouping is a better information architecture than the original flat menu,
the dashboard now has a first-run starter checklist plus optional synthetic
sample workspace, and contact detail now starts to behave like a Customer 360
workspace with open-deal, pipeline, next-follow-up, recent-activity, and
attention-status summary signals. The product still undersells its foundation
because several core CRM jobs remain shallow or absent. Comparable CRMs make
sales work feel richer through leads, inbox and communication history, calendar
views, reports, saved views, automation, guided onboarding, and clearer account
and deal workspaces.

The next phase should therefore be a product-depth phase, not another foundation
phase. The priority is to make 900CRM feel useful in the first 10 minutes for a
real small business owner.

## Current Menu Review

| Area | Current state | Product issue |
|---|---|---|
| Dashboard | KPI cards, reports, activity feed, quick actions, first-run starter checklist, optional synthetic sample workspace | Useful start, but not yet a true command center. No saved goals, no forecast view, and no "what needs attention" queue. |
| Contacts | Search, type filter, duplicate review, import/export, custom-field filtering, contact detail Customer 360 summary | Contacts and lead/account work are blended. The detail view now has a useful at-a-glance summary, but there is no lead capture stage, saved segment, list view preset, or full interaction timeline. |
| Organizations | List, create/edit, notes/tags, contact linking | Important, but weaker than account management in competing CRMs. No account health, open deals summary, recent activity, owner, or next step at list level. |
| Pipeline | Kanban by stage, custom-field filter, deal cards | Good foundation. Needs drag/drop confidence, stage conversion metrics, stale deal detection, forecast view, and deal detail workspace. |
| Activities | Task/call/meeting/email list with filters and completion | Useful but should become a calendar/task center with due buckets, day/week views, reminders, and relationship context. |
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
| Dashboard/reporting | 1 | 2 |
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

### 2. Contacts are becoming a customer workspace, but accounts are still thin

The contact detail page exists, notes/tags/custom fields exist, and Sprint 102
added a contact-level Customer 360 summary for open deals, open pipeline value,
next follow-up, recent activity, and attention status. That makes contacts feel
less like a static address book. The product still needs the fuller "customer
360" pattern: profile, timeline, linked organization, activities, notes, tags,
files/links, and next action in one place. Organizations also still lack a real
account workspace.

Recommended outcome: richer contact and organization detail workspaces before
adding unrelated modules.

### 3. Pipeline lacks sales guidance

The Kanban board exists, but a salesperson needs to know which deals are stale,
which stage is leaking, what is forecast to close, and what next action is due.

Recommended outcome: pipeline stage metrics, stale deal badges, missing-next-step
warnings, expected close date/forecast view, and a deal detail drawer.

### 4. Activities need a calendar/task center

The current activity list is functional, but daily users expect "today",
"overdue", "this week", and calendar-like planning. Pipedrive/Odoo-style
follow-up discipline is central to CRM value.

Recommended outcome: Activities becomes a day/week workbench with due buckets,
quick reschedule, complete/snooze, and relationship context.

### 5. Admin/safety items crowd primary navigation

Audit Log and Pending Actions are important, but they are not daily top-level
CRM work for most users. They make the app feel technical.

Recommended outcome: group these under a Review/Admin section, or show them
only when counts require attention.

### 6. Settings is overloaded

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

3. **Customer 360 Workspaces** - started for contact detail summary
   - Added contact detail open-deal, open-pipeline, next-follow-up,
     recent-activity, and attention-status summary.
   - Remaining work: organization/account detail workspace, fuller interaction
     timeline, linked records, files/links, and richer record routing.
   - Make global search route directly into useful record views.

4. **Pipeline Depth**
   - Add deal detail drawer/page.
   - Add stale deal detection, missing next activity warning, stage aging, close
     date, forecast amount, and stage conversion summaries.

5. **Activities Calendar and Follow-up Center**
   - Add today/overdue/this-week views.
   - Add quick reschedule, snooze, complete, and relationship breadcrumbs.
   - Consider a lightweight calendar view without requiring online calendar
     sync.

6. **Reports Hub**
   - Move dashboard reports into a Reports page.
   - Add saved filters, pipeline conversion, activity completion, stale deals,
     source/owner breakdown where supported, and exportable report snapshots.

7. **Local Automation Lite**
   - Add simple local rules: when deal moves stage, create follow-up; when
     activity is overdue, flag; when contact is created from import, tag/source.
   - Keep it local and explicit; no cloud automation dependency.

8. **Alpha Packaging and Smoke Evidence**
   - Continue release packaging work once GitHub Actions billing/spending-limit
     is resolved.
   - Produce and test Windows, macOS, and Linux artifacts.

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
