---
gsd_state_version: 1.0
milestone: v1.0
milestone_name: milestone
status: in-progress
stopped_at: Completed 04-01-PLAN.md
last_updated: "2026-03-09T02:08:00Z"
last_activity: 2026-03-09 -- Completed plan 04-01 (viability polish + schedule enhancements)
progress:
  total_phases: 4
  completed_phases: 3
  total_plans: 8
  completed_plans: 8
  percent: 100
---

# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-03-08)

**Core value:** Take the complicated scheduling math out of starting seeds indoors -- tell the app what you want to grow, and it tells you when to plant, transplant, and expect harvest.
**Current focus:** Phase 4 in progress. Viability polish and schedule enhancements complete.

## Current Position

Phase: 4 of 4 (Polish + Differentiators)
Plan: 1 of 1 in current phase (done)
Status: Phase 4 Plan 1 complete
Last activity: 2026-03-09 -- Completed plan 04-01 (viability polish + schedule enhancements)

Progress: [██████████] 100%

## Performance Metrics

**Velocity:**
- Total plans completed: 8
- Average duration: 5min
- Total execution time: 0.68 hours

**By Phase:**

| Phase | Plans | Total | Avg/Plan |
|-------|-------|-------|----------|
| 01-foundation-scraping | 3/3 | 21min | 7min |
| 02-seed-inventory-viability | 2/2 | 6min | 3min |
| 03-season-planning-schedule-views | 2/2 | 12min | 6min |
| 04-polish-differentiators | 1/1 | 3min | 3min |

**Recent Trend:**
- Last 5 plans: 02-01 (3min), 02-02 (3min), 03-01 (9min), 03-02 (3min), 04-01 (3min)
- Trend: stable

*Updated after each plan completion*

## Accumulated Context

### Decisions

Decisions are logged in PROJECT.md Key Decisions table.
Recent decisions affecting current work:

- [Roadmap]: 4-phase structure following scraper-first dependency chain per research findings
- [Roadmap]: Core viability (VIAB-01, VIAB-02) in Phase 2; polish viability features (VIAB-03-05) deferred to Phase 4
- [01-01]: WAL mode set via PRAGMA on pool connection rather than in migration SQL (SQLite cannot change journal mode inside a transaction)
- [01-01]: HTMX 2.0.4 served locally from /static/ for offline support (INFR-03)
- [01-02]: HTML growing details uses multi-strategy best-effort extraction with graceful None fallback; raw HTML stored for future refinement
- [01-02]: DuplicateSeed is a distinct AppError variant (HTTP 409) for clear UX handling
- [01-03]: Collection-prefixed URLs (/collections/.../products/...) accepted alongside direct /products/ URLs for BI site compatibility
- [02-01]: Viability computed at render time (not stored in DB) since it depends on current year
- [02-01]: Linear decline viability model (100% at age 0, 0% at max_years) -- adequate for gardening helper
- [02-01]: LazyLock HashMap for species lookup table (80+ entries, no external crate needed)
- [02-01]: SubCat prefix normalization for Botanical Interests tag format compatibility
- [02-02]: Separate seed_purchases table instead of purchase_year column on seeds -- supports multiple lots per seed with independent viability tracking
- [02-02]: Migration 003 auto-migrates existing purchase_year data to seed_purchases
- [02-02]: Seed list shows newest purchase viability; detail page shows all lots individually
- [03-01]: Planting parser uses string search (not regex crate) since BI patterns are predictable
- [03-01]: Two-phase warm-season pattern stores raw weeks values; calculator combines transplant-relative and frost-relative offsets
- [03-01]: Toggle button placed outside <a> tag in seed list with stopPropagation to prevent navigation
- [03-02]: Layout refactored to layout_with_nav(title, active_nav, content) for proper nav highlighting
- [03-02]: Timeline period bars extend 6 weeks past action date for visual growing season indication
- [03-02]: Removed disabled Inventory nav link entirely since no inventory page exists
- [04-01]: Viability color tiers use fixed percentage ranges (75-100 green, 50-74 yellow, 25-49 orange, 0-24 red)
- [04-01]: Sow multiplier calculated as 100/percentage, None for >= 90% or 0%
- [04-01]: This Week view uses 14-day forward window plus overdue from current week (Monday)

### Pending Todos

None yet.

### Blockers/Concerns

- [Research]: Botanical Interests HTML structure not yet validated -- scraper is highest-risk component. Must validate against real pages at start of Phase 1.
- [Research]: Planting data is unstructured free-form text requiring regex parsing that varies by product category.

## Session Continuity

Last session: 2026-03-09T02:04:31Z
Stopped at: Completed 04-01-PLAN.md
Resume file: .planning/phases/04-polish-differentiators/04-01-SUMMARY.md
