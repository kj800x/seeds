---
phase: 01-foundation-scraping
plan: 03
subsystem: ui
tags: [axum, maud, htmx, routes, templates, seed-detail, form-handling]

# Dependency graph
requires:
  - phase: 01-foundation-scraping/01-01
    provides: Axum server, SQLite schema, Maud layout, AppState, DB queries, AppError, CSS theme
  - phase: 01-foundation-scraping/01-02
    provides: scrape_and_save function, DuplicateSeed error variant, image downloading
provides:
  - Seed detail page with hero image, structured sections, and collapsible original text
  - Add-seed POST handler with URL validation, duplicate detection, and HTMX redirect
  - Seed detail GET handler with 404 handling
  - Full end-to-end scraping flow from UI form to stored/displayed seed data
affects: [02-scheduling]

# Tech tracking
tech-stack:
  added: []
  patterns: [HTMX form with hx-post and HX-Redirect response, details/summary for collapsible sections, conditional Maud rendering for optional fields]

key-files:
  created:
    - seeds-rs/src/routes/seeds.rs
    - seeds-rs/src/templates/seed_detail.rs
  modified:
    - seeds-rs/src/main.rs
    - seeds-rs/src/routes/mod.rs
    - seeds-rs/src/templates/mod.rs
    - seeds-rs/src/scraper/fetcher.rs
    - seeds-rs/static/style.css

key-decisions:
  - "Collection-prefixed URLs (/collections/.../products/...) accepted alongside direct /products/ URLs"

patterns-established:
  - "HTMX form pattern: hx-post with HX-Redirect header on success, inline HTML fragment on error"
  - "Detail page pattern: hero image, structured sections (skip if empty), collapsible original text via details/summary"
  - "URL validation: allow both /products/ and /collections/.../products/ paths from Botanical Interests"

requirements-completed: [SCRP-01, SCRP-04]

# Metrics
duration: 12min
completed: 2026-03-08
---

# Phase 1 Plan 3: Seed Routes and UI Wiring Summary

**Seed detail page with hero image, structured growing sections, HTMX add-seed form with duplicate detection and collection URL support**

## Performance

- **Duration:** 12 min (across two sessions with human verification checkpoint)
- **Started:** 2026-03-08T22:00:00Z
- **Completed:** 2026-03-08T22:12:00Z
- **Tasks:** 2
- **Files modified:** 7

## Accomplishments
- Full end-to-end scraping flow: paste BI URL in form, see spinner, get redirected to detail page
- Seed detail page with hero image, structured growing/planting/harvest/about sections, and collapsible original text
- Add-seed handler with URL validation, duplicate detection (inline message with link), and error display
- Seed list on home page with clickable rows linking to detail pages
- Warm/earthy CSS styling for detail page, badges, collapsible sections, and error messages

## Task Commits

Each task was committed atomically:

1. **Task 1: Seed routes, detail template, and router wiring** - `6201137` (feat)
2. **Bug fix: Accept collection-prefixed URLs** - `3bd015a` (fix)
3. **Task 2: Verify full scraping flow end-to-end** - human-verify checkpoint, approved by user

## Files Created/Modified
- `seeds-rs/src/routes/seeds.rs` - Add-seed POST handler and seed detail GET handler (86 lines)
- `seeds-rs/src/templates/seed_detail.rs` - Detail page with hero image, structured sections, collapsible text (139 lines)
- `seeds-rs/src/main.rs` - Router wiring for /seeds/{id} and /seeds/add routes
- `seeds-rs/src/routes/mod.rs` - Export seeds route module
- `seeds-rs/src/templates/mod.rs` - Export seed_detail template module
- `seeds-rs/src/scraper/fetcher.rs` - URL validation fix for collection-prefixed URLs
- `seeds-rs/static/style.css` - Detail page, badge, collapsible, and error styles (193 lines added)

## Decisions Made
- Collection-prefixed URLs (/collections/beans/products/...) accepted alongside direct /products/ URLs, as Botanical Interests uses both URL patterns depending on navigation path

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Collection-prefixed URLs rejected by URL validation**
- **Found during:** Task 2 (human verification checkpoint)
- **Issue:** URLs like /collections/beans/products/... were being rejected because validation only checked for /products/ prefix, but BI site uses collection-prefixed URLs when navigating from category pages
- **Fix:** Updated URL validation in fetcher.rs to accept both /products/ and /collections/.../products/ URL patterns
- **Files modified:** seeds-rs/src/scraper/fetcher.rs
- **Verification:** User verified collection URLs work correctly after fix
- **Committed in:** `3bd015a`

---

**Total deviations:** 1 auto-fixed (1 bug)
**Impact on plan:** Fix was necessary for real-world URL patterns. No scope creep.

## Issues Encountered
None beyond the auto-fixed URL validation bug above.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Phase 1 (Foundation + Scraping) is now complete
- Full scraping pipeline working: URL input -> fetch -> parse -> store -> display
- SQLite persistence verified across server restarts
- Image downloading and serving operational
- Ready for Phase 2: Scheduling features (frost dates, planting calendar)

## Self-Check: PASSED

All files and commits verified.

---
*Phase: 01-foundation-scraping*
*Completed: 2026-03-08*
