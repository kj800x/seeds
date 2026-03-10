---
phase: 02-seed-inventory-viability
plan: 02
subsystem: ui, database
tags: [htmx, maud, sqlite, axum, viability, seed-purchases]

requires:
  - phase: 02-seed-inventory-viability
    provides: viability estimation module, seed CRUD queries, inventory schema
  - phase: 01-foundation-scraping
    provides: scraper, seed model, routing infrastructure
provides:
  - Seed purchase tracking with multiple lots per seed
  - Per-lot viability display in seed list and detail page
  - HTMX CRUD for purchase records (add/edit/delete)
  - Add-seed flow creates seed + optional first purchase
affects: [03-planting-schedule, 04-polish-extras]

tech-stack:
  added: []
  patterns: [multi-lot purchase tracking, HTMX table row inline edit, seed_purchases FK relationship]

key-files:
  created:
    - seeds-rs/migrations/003_seed_purchases.sql
  modified:
    - seeds-rs/src/db/models.rs
    - seeds-rs/src/db/queries.rs
    - seeds-rs/src/routes/seeds.rs
    - seeds-rs/src/routes/home.rs
    - seeds-rs/src/templates/home.rs
    - seeds-rs/src/templates/seed_detail.rs
    - seeds-rs/src/main.rs
    - seeds-rs/src/scraper/mod.rs
    - seeds-rs/static/style.css

key-decisions:
  - "Separate seed_purchases table instead of purchase_year column on seeds -- supports multiple lots per seed with independent viability tracking"
  - "Migration 003 auto-migrates existing purchase_year data to seed_purchases table"
  - "Deprecated purchase_year/notes columns left in seeds table (SQLite lacks DROP COLUMN in older versions) but no longer written to"
  - "Seed list shows newest purchase year viability; detail page shows all lots"

patterns-established:
  - "HTMX table row editing: GET .../edit returns tr replacement, PUT returns full section"
  - "Purchase-level viability: viability computed per lot, not per seed"

requirements-completed: [INVT-01, INVT-02, INVT-03, INVT-04, INVT-05, VIAB-01, VIAB-02]

duration: 3min
completed: 2026-03-08
---

# Phase 2 Plan 2: Inventory UI & Viability Summary

**Multi-lot seed purchase tracking with per-lot viability display, HTMX inline CRUD, and purchase history table on detail page**

## Performance

- **Duration:** 3 min
- **Started:** 2026-03-08T23:48:06Z
- **Completed:** 2026-03-08T23:51:30Z
- **Tasks:** 3 (tasks 1-2 from prior session, task 3 reworked per user feedback)
- **Files modified:** 9

## Accomplishments
- Seed purchase history tracked in separate `seed_purchases` table supporting multiple lots per seed
- Detail page shows purchase history table with per-lot viability percentages and HTMX inline edit/delete
- Seed list shows newest purchase year and viability; multi-lot seeds show lot count
- Add-seed form creates seed + optional first purchase record in one flow

## Task Commits

Each task was committed atomically:

1. **Task 1: Add purchase year to add-seed form and viability to seed list** - `2bf8076` (feat)
2. **Task 2: Edit/delete on detail page with HTMX and viability display** - `7d20514` (feat)
3. **Task 3a: Backend - seed_purchases table, models, queries** - `d5cb984` (feat)
4. **Task 3b: Frontend - multi-purchase UI with per-lot viability** - `f104972` (feat)

## Files Created/Modified
- `seeds-rs/migrations/003_seed_purchases.sql` - New table for purchase lots with data migration
- `seeds-rs/src/db/models.rs` - Added SeedPurchase struct
- `seeds-rs/src/db/queries.rs` - CRUD for seed_purchases, newest/count aggregation queries
- `seeds-rs/src/routes/seeds.rs` - Purchase CRUD handlers, updated add_seed to create purchase
- `seeds-rs/src/routes/home.rs` - Passes purchase aggregates to home template
- `seeds-rs/src/templates/home.rs` - Shows newest purchase year, lot count, viability per seed
- `seeds-rs/src/templates/seed_detail.rs` - Purchase history table with inline edit/add/delete
- `seeds-rs/src/main.rs` - Routes for purchase CRUD endpoints
- `seeds-rs/src/scraper/mod.rs` - Removed purchase_year/notes from NewSeed
- `seeds-rs/static/style.css` - Purchases table, inline edit, btn-sm styles

## Decisions Made
- Used separate `seed_purchases` table instead of purchase_year column on seeds, per user feedback that same seed species may be purchased multiple years
- Migration 003 automatically migrates existing data from seeds.purchase_year to seed_purchases
- Left deprecated columns in seeds table rather than attempting SQLite ALTER TABLE DROP COLUMN
- Seed list shows viability based on newest purchase year; detail page shows all lots individually

## Deviations from Plan

### Design Change (User-Requested at Checkpoint)

**1. [Rule 4 - Architectural] Separate seed_purchases table for multi-lot tracking**
- **Found during:** Task 3 checkpoint review
- **Issue:** Original plan had purchase_year as single column on seeds table. User identified that the same seed type may be purchased across multiple years and each lot needs independent viability tracking.
- **Fix:** Created seed_purchases table (migration 003), added SeedPurchase model, CRUD queries, and reworked all UI to display/manage multiple purchases per seed
- **Files modified:** All 9 files listed above
- **Verification:** cargo build succeeds
- **Committed in:** d5cb984 (backend), f104972 (frontend)

---

**Total deviations:** 1 user-requested architectural change
**Impact on plan:** Significant rework of both backend and frontend, but aligned with user's actual domain needs. Same requirements fulfilled with better data model.

## Issues Encountered
None - changes compiled cleanly on first attempt.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Inventory CRUD complete with multi-lot purchase tracking
- Viability estimation working per-lot
- Ready for Phase 3 (planting schedule) which will use purchase/viability data

## Self-Check: PASSED

All 11 files verified present. All 4 commits (2bf8076, 7d20514, d5cb984, f104972) verified in git history.

---
*Phase: 02-seed-inventory-viability*
*Completed: 2026-03-08*
