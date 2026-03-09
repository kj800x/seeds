---
phase: 03-season-planning-schedule-views
plan: 01
subsystem: schedule
tags: [chrono, NaiveDate, planting-parser, frost-dates, htmx, maud]

# Dependency graph
requires:
  - phase: 02-seed-inventory-viability
    provides: seeds table with planting_instructions field, seed list UI
provides:
  - schedule module with planting text parser and date calculator
  - season_plans DB table with seed_id + year unique constraint
  - PlantingTiming, PlantingAction, ActionType types
  - Season plan toggle UI on seed list via HTMX
  - Season plan CRUD queries (list, toggle, planned_seed_ids)
affects: [03-02-schedule-views, schedule-display, timeline-views]

# Tech tracking
tech-stack:
  added: []
  patterns: [schedule-module, planting-text-parser, frost-date-calculator, htmx-toggle-button]

key-files:
  created:
    - seeds-rs/migrations/004_season_plans.sql
    - seeds-rs/src/schedule/mod.rs
    - seeds-rs/src/schedule/parser.rs
    - seeds-rs/src/schedule/calculator.rs
  modified:
    - seeds-rs/src/db/models.rs
    - seeds-rs/src/db/queries.rs
    - seeds-rs/src/main.rs
    - seeds-rs/src/routes/seeds.rs
    - seeds-rs/src/routes/home.rs
    - seeds-rs/src/templates/home.rs
    - seeds-rs/static/style.css

key-decisions:
  - "Planting parser uses string search (not regex crate) since BI patterns are predictable"
  - "Two-phase warm-season pattern stores raw weeks values; calculator combines transplant-relative and frost-relative offsets"
  - "Toggle button placed outside <a> tag in seed list with stopPropagation to prevent navigation"

patterns-established:
  - "Schedule module: parser extracts timing, calculator resolves to calendar dates"
  - "DRY toggle button: plan_toggle_button() helper used in both list render and POST response"
  - "Computed-at-render dates: same pattern as viability, not stored in DB"

requirements-completed: [PLAN-01, PLAN-02, PLAN-03, PLAN-04, PLAN-05]

# Metrics
duration: 9min
completed: 2026-03-09
---

# Phase 3 Plan 1: Schedule Engine Summary

**Planting text parser extracting frost-relative timing from BI instructions, date calculator producing calendar dates via chrono, and HTMX toggle UI for season plan selection**

## Performance

- **Duration:** 9 min
- **Started:** 2026-03-09T01:12:56Z
- **Completed:** 2026-03-09T01:22:17Z
- **Tasks:** 2
- **Files modified:** 11

## Accomplishments
- Schedule parser handles 5 planting text patterns: cool-season before-frost, warm-season after-frost, two-phase warm-season (indoor + transplant), long-start indoor, and direct sow
- Date calculator computes NaiveDate values relative to Halifax MA May 10 last frost, correctly handling warm-season two-phase crops where indoor start is relative to transplant date
- Season plan toggle on seed list adds/removes seeds from current year's plan via HTMX without page reload
- 15 unit tests covering parser patterns and calculator date arithmetic, 41 total tests passing

## Task Commits

Each task was committed atomically:

1. **Task 1: Schedule engine -- DB migration, parser, calculator** - `d2e8e6f` (feat)
2. **Task 2: Season plan toggle UI on seed list** - `25a0542` (feat)

## Files Created/Modified
- `seeds-rs/migrations/004_season_plans.sql` - Season plans table with UNIQUE(seed_id, year)
- `seeds-rs/src/schedule/mod.rs` - Public re-exports for schedule module
- `seeds-rs/src/schedule/parser.rs` - PlantingTiming struct and parse_planting_timing function with 9 tests
- `seeds-rs/src/schedule/calculator.rs` - PlantingAction, ActionType, generate_schedule with 6 tests
- `seeds-rs/src/db/models.rs` - Added SeasonPlan model
- `seeds-rs/src/db/queries.rs` - Added season plan CRUD (list, toggle, planned_seed_ids)
- `seeds-rs/src/routes/seeds.rs` - Added toggle_plan POST handler
- `seeds-rs/src/routes/home.rs` - Fetches planned seed IDs for current year
- `seeds-rs/src/templates/home.rs` - Added plan_toggle_button helper and toggle in seed list
- `seeds-rs/src/main.rs` - Added schedule module and toggle route
- `seeds-rs/static/style.css` - Toggle button styles with active/inactive states

## Decisions Made
- Used string search methods instead of regex crate since BI planting text patterns are predictable and limited
- Two-phase warm-season pattern (e.g., tomato) stores raw "weeks before transplanting" value; calculator computes transplant date from frost date then subtracts indoor weeks
- Toggle button rendered outside the `<a>` tag in seed-item `<li>` with event.stopPropagation() to prevent click-through to seed detail

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered
None

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Schedule module ready for plan 03-02 to build schedule list and timeline views
- season_plans table enables querying planned seeds for schedule rendering
- PlantingTiming + generate_schedule pipeline provides all data needed for schedule views

---
*Phase: 03-season-planning-schedule-views*
*Completed: 2026-03-09*
