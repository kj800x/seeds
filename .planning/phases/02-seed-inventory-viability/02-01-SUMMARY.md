---
phase: 02-seed-inventory-viability
plan: 01
subsystem: database, viability
tags: [sqlite, chrono, viability, inventory, crud, rust]

# Dependency graph
requires:
  - phase: 01-foundation-scraping
    provides: Seed model, seeds table, insert_seed query, main.rs module structure
provides:
  - purchase_year and notes columns on seeds table (migration 002)
  - update_seed and delete_seed query functions
  - ViabilityEstimate struct and estimate_viability function
  - Species lookup table with 80+ entries and SubCat prefix normalization
affects: [02-seed-inventory-viability, 03-planting-schedule, 04-polish]

# Tech tracking
tech-stack:
  added: [chrono]
  patterns: [viability-as-computation, species-lookup-table, linear-decline-model]

key-files:
  created:
    - seeds-rs/migrations/002_inventory.sql
    - seeds-rs/src/viability/mod.rs
    - seeds-rs/src/viability/lookup.rs
  modified:
    - seeds-rs/Cargo.toml
    - seeds-rs/src/db/models.rs
    - seeds-rs/src/db/queries.rs
    - seeds-rs/src/scraper/mod.rs
    - seeds-rs/src/main.rs

key-decisions:
  - "Viability computed at render time (not stored in DB) since it depends on current year"
  - "Linear decline model (100% at age 0, 0% at max_years) -- adequate for gardening helper"
  - "LazyLock HashMap for species lookup -- no external crate needed (stable since Rust 1.80)"
  - "SubCat prefix normalization handles Botanical Interests tag format"

patterns-established:
  - "Viability as pure computation: estimate_viability(subcategory, category, purchase_year) -> Option<ViabilityEstimate>"
  - "Species lookup with subcategory-first, category-fallback, then default (2 years)"
  - "Test helper with explicit current_year parameter for deterministic time-dependent tests"

requirements-completed: [INVT-03, INVT-04, INVT-05, VIAB-01, VIAB-02]

# Metrics
duration: 3min
completed: 2026-03-08
---

# Phase 2 Plan 1: Inventory & Viability Data Foundation Summary

**SQLite migration for purchase_year/notes, update/delete queries, and viability estimation module with 80+ species lookup table using linear decline model**

## Performance

- **Duration:** 3 min
- **Started:** 2026-03-08T23:26:20Z
- **Completed:** 2026-03-08T23:29:04Z
- **Tasks:** 2
- **Files modified:** 8

## Accomplishments
- Database migration adds purchase_year and notes columns to seeds table
- Full inventory CRUD with update_seed and delete_seed query functions
- Viability estimation module with 80+ species lookup table covering vegetables, herbs, and flowers
- 15 unit tests covering species lookup, viability calculation, null handling, and SubCat prefix normalization

## Task Commits

Each task was committed atomically:

1. **Task 1: Database migration and model updates** - `f92e833` (feat)
2. **Task 2: Viability estimation module with species lookup** - `77138cf` (feat)

## Files Created/Modified
- `seeds-rs/migrations/002_inventory.sql` - Adds purchase_year INTEGER and notes TEXT columns
- `seeds-rs/src/viability/mod.rs` - ViabilityEstimate struct, estimate_viability function, 8 tests
- `seeds-rs/src/viability/lookup.rs` - Species-to-max-years HashMap (80+ entries), lookup_max_years function, 7 tests
- `seeds-rs/Cargo.toml` - Added chrono dependency
- `seeds-rs/src/db/models.rs` - Added purchase_year and notes to Seed struct
- `seeds-rs/src/db/queries.rs` - Added fields to NewSeed, updated insert_seed, added update_seed and delete_seed
- `seeds-rs/src/scraper/mod.rs` - Updated NewSeed construction with None defaults for new fields
- `seeds-rs/src/main.rs` - Registered viability module

## Decisions Made
- Viability computed at render time (not stored in DB) since it depends on current year
- Linear decline model chosen for simplicity -- adequate for a gardening helper app
- Used std::sync::LazyLock for species table (stable, no external crate)
- SubCat prefix normalization handles "SubCat - Tomato" -> "tomato" for BI tag compatibility
- Test helper accepts explicit current_year parameter for deterministic tests

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Updated scraper NewSeed construction**
- **Found during:** Task 1 (model updates)
- **Issue:** Scraper mod.rs constructs NewSeed without purchase_year and notes fields, causing compilation failure
- **Fix:** Added `purchase_year: None, notes: None` to NewSeed literal in scraper/mod.rs
- **Files modified:** seeds-rs/src/scraper/mod.rs
- **Verification:** cargo check passes
- **Committed in:** f92e833 (Task 1 commit)

---

**Total deviations:** 1 auto-fixed (1 blocking)
**Impact on plan:** Necessary fix to maintain compilation. No scope creep.

## Issues Encountered
None

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Data layer complete: Seed model has purchase_year/notes, CRUD queries ready
- Viability module ready to be called from templates/routes in Plan 02
- Plan 02 (HTMX edit/delete UI and viability display) can proceed

---
*Phase: 02-seed-inventory-viability*
*Completed: 2026-03-08*
