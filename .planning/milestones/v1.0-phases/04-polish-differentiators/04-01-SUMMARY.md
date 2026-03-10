---
phase: 04-polish-differentiators
plan: 01
subsystem: ui
tags: [viability, color-coding, schedule, print-css, htmx]

requires:
  - phase: 02-seed-inventory-viability
    provides: ViabilityEstimate struct and estimate_viability function
  - phase: 03-season-planning-schedule-views
    provides: schedule page, action list rendering, timeline
provides:
  - Color-coded viability tiers (green/yellow/orange/red) in seed list and detail
  - Warning messages for low-viability and expired seeds
  - Sow multiplier suggestions for reduced germination
  - This Week schedule view with 14-day window filtering
  - Print stylesheet for clean action list printing
  - Schedule tab navigation (Full Season / This Week)
affects: []

tech-stack:
  added: []
  patterns:
    - "Viability helper methods on ViabilityEstimate for display logic"
    - "HTMX tab navigation with hx-target and hx-push-url for schedule views"
    - "CSS color tiers via match on percentage ranges"

key-files:
  created: []
  modified:
    - seeds-rs/src/viability/mod.rs
    - seeds-rs/src/templates/home.rs
    - seeds-rs/src/templates/seed_detail.rs
    - seeds-rs/src/templates/schedule.rs
    - seeds-rs/src/routes/schedule.rs
    - seeds-rs/src/main.rs
    - seeds-rs/static/style.css

key-decisions:
  - "Viability color tiers use fixed percentage ranges (75-100 green, 50-74 yellow, 25-49 orange, 0-24 red)"
  - "Sow multiplier calculated as 100/percentage, returns None for >= 90% or 0%"
  - "This Week view uses 14-day forward window plus overdue from current week"

patterns-established:
  - "ViabilityEstimate methods: color_tier(), warning_message(), sow_multiplier()"
  - "Schedule tab pattern: HTMX tabs with hx-target='.schedule-content' and push-url"

requirements-completed: [VIAB-03, VIAB-04, VIAB-05]

duration: 3min
completed: 2026-03-09
---

# Phase 4 Plan 1: Viability Polish and Schedule Enhancements Summary

**Color-coded viability tiers with warnings/sow-multiplier on detail page, plus This Week schedule view with HTMX tabs and print stylesheet**

## Performance

- **Duration:** 3 min
- **Started:** 2026-03-09T02:04:31Z
- **Completed:** 2026-03-09T02:07:40Z
- **Tasks:** 2
- **Files modified:** 7

## Accomplishments
- Added color_tier(), warning_message(), sow_multiplier() methods to ViabilityEstimate with full test coverage (14 new tests)
- Seed list viability spans now color-coded green/yellow/orange/red based on percentage tier
- Seed detail page shows warning banner for expired or last-year seeds, and sow multiplier suggestion for reduced viability
- Added /schedule/week route with 14-day window + overdue filtering and empty state with next action hint
- Added Full Season / This Week tab navigation with HTMX
- Added @media print stylesheet hiding nav, chrome, and timeline for clean printable action lists
- All 55 tests pass, project compiles clean

## Task Commits

Each task was committed atomically:

1. **Task 1: Add viability helper methods and color-coded display** - `127bdef` (test: RED phase), `c2836d9` (feat: GREEN phase)
2. **Task 2: This week schedule view and print stylesheet** - `498f901` (feat)

## Files Created/Modified
- `seeds-rs/src/viability/mod.rs` - Added color_tier(), warning_message(), sow_multiplier() impl + 14 unit tests
- `seeds-rs/src/templates/home.rs` - Color-coded viability spans using est.color_tier()
- `seeds-rs/src/templates/seed_detail.rs` - Warning banner + sow multiplier display after purchases section
- `seeds-rs/src/templates/schedule.rs` - Tab navigation, schedule-content wrapper, this_week_template
- `seeds-rs/src/routes/schedule.rs` - this_week handler with 14-day window + overdue filtering
- `seeds-rs/src/main.rs` - Registered /schedule/week route
- `seeds-rs/static/style.css` - Viability color tiers, warning/suggestion styling, schedule tabs, print stylesheet

## Decisions Made
- Viability color tiers use fixed percentage ranges matching plan spec (75-100 green, 50-74 yellow, 25-49 orange, 0-24 red)
- Sow multiplier calculated as 100.0/percentage with None for >= 90% (no compensation needed) and 0% (seeds dead)
- This Week view includes overdue actions from current week (Monday) through 14 days ahead
- Empty state on This Week shows the next upcoming action beyond the window

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered
None

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Phase 4 Plan 1 complete with all viability polish and schedule enhancements
- Ready for remaining Phase 4 plans if any exist

---
*Phase: 04-polish-differentiators*
*Completed: 2026-03-09*
