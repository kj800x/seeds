---
phase: 01-foundation-scraping
plan: 01
subsystem: infra
tags: [axum, maud, htmx, sqlx, sqlite, tower-http, rust]

# Dependency graph
requires: []
provides:
  - Axum web server with SQLite connection pool and migrations
  - Maud layout shell (header, nav, content area) for all pages
  - SQLite schema with seeds and seed_images tables
  - Static file serving for CSS and HTMX JS
  - DB query functions for seed CRUD operations
  - AppError type with HTML error responses
  - Warm/earthy CSS theme with custom properties
affects: [01-02, 01-03]

# Tech tracking
tech-stack:
  added: [axum 0.8, maud 0.27, sqlx 0.8, reqwest 0.13, scraper 0.22, tokio 1, tower-http 0.6, serde 1, tracing 0.1, htmx 2.0.4]
  patterns: [MASH stack (Maud+Axum+SQLx+HTMX), shared AppState with SqlitePool, Maud layout composition, ServeDir for static files]

key-files:
  created:
    - seeds-rs/Cargo.toml
    - seeds-rs/migrations/001_initial.sql
    - seeds-rs/src/main.rs
    - seeds-rs/src/db/models.rs
    - seeds-rs/src/db/queries.rs
    - seeds-rs/src/error.rs
    - seeds-rs/src/templates/layout.rs
    - seeds-rs/src/templates/home.rs
    - seeds-rs/src/routes/home.rs
    - seeds-rs/static/style.css
    - seeds-rs/static/htmx.min.js
  modified: []

key-decisions:
  - "WAL mode set via PRAGMA on pool connection rather than in migration SQL (SQLite cannot change journal mode inside a transaction)"
  - "HTMX 2.0.4 served locally from /static/ for offline support (INFR-03)"

patterns-established:
  - "AppState pattern: Clone struct with SqlitePool + PathBuf data_dir, passed via Axum with_state"
  - "Layout composition: layout(title, content) function wrapping all pages in consistent shell"
  - "Route handler pattern: async fn handler(State(state): State<AppState>) -> Result<Markup, AppError>"
  - "Error handling: AppError enum implementing IntoResponse with HTML error pages"

requirements-completed: [INFR-01, INFR-02, INFR-03]

# Metrics
duration: 6min
completed: 2026-03-08
---

# Phase 1 Plan 1: Project Setup Summary

**Axum server with SQLite (WAL mode), Maud layout shell, HTMX 2.0 local serving, and warm/earthy CSS dashboard theme**

## Performance

- **Duration:** 6 min
- **Started:** 2026-03-08T21:39:08Z
- **Completed:** 2026-03-08T21:45:11Z
- **Tasks:** 2
- **Files modified:** 18

## Accomplishments
- Bootable Axum server serving styled dashboard at localhost:3000
- SQLite database with seeds and seed_images tables, WAL mode enabled
- Complete Maud layout with header, nav (Seeds active, Inventory/Schedule disabled), and content area
- Warm/earthy vanilla CSS theme with custom properties, responsive design
- HTMX 2.0.4 served locally for offline support
- DB query layer with full CRUD stubs ready for scraper integration

## Task Commits

Each task was committed atomically:

1. **Task 1: Project dependencies, SQLite schema, and data models** - `2226fca` (feat)
2. **Task 2: Axum server, Maud layout, static files, and landing page** - `7724d95` (feat)

## Files Created/Modified
- `seeds-rs/Cargo.toml` - All project dependencies (axum, maud, sqlx, reqwest, scraper, etc.)
- `seeds-rs/migrations/001_initial.sql` - SQLite schema with seeds and seed_images tables
- `seeds-rs/src/main.rs` - Server startup, pool init, WAL mode, router assembly, static serving
- `seeds-rs/src/db/models.rs` - Seed, SeedImage, AppState structs with sqlx::FromRow
- `seeds-rs/src/db/queries.rs` - list_seeds, get_seed, insert_seed, find_seed_by_handle, insert_image
- `seeds-rs/src/db/mod.rs` - DB module re-exports
- `seeds-rs/src/error.rs` - AppError enum (DbError, ScraperError, NotFound) with HTML IntoResponse
- `seeds-rs/src/templates/layout.rs` - Maud layout shell with header, nav, content wrapper
- `seeds-rs/src/templates/home.rs` - Home page with add-seed form and seed list
- `seeds-rs/src/templates/mod.rs` - Template module re-exports
- `seeds-rs/src/routes/home.rs` - Home route handler querying seeds and rendering page
- `seeds-rs/src/routes/mod.rs` - Route module re-exports
- `seeds-rs/static/style.css` - Warm/earthy CSS with custom properties, responsive layout
- `seeds-rs/static/htmx.min.js` - HTMX 2.0.4 minified (local copy for offline)

## Decisions Made
- WAL mode set via PRAGMA on pool connection rather than in migration SQL, because SQLite cannot change journal mode inside a transaction (sqlx runs migrations transactionally)
- HTMX 2.0.4 downloaded and served locally from /static/ to satisfy INFR-03 offline requirement
- Removed nested .git directory from seeds-rs/ to allow proper tracking in parent repo

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] WAL PRAGMA moved out of migration**
- **Found during:** Task 2 (server startup verification)
- **Issue:** PRAGMA journal_mode=WAL in migration SQL caused "cannot change into wal mode from within a transaction" error because sqlx runs migrations inside transactions
- **Fix:** Removed PRAGMA from migration, added sqlx::query("PRAGMA journal_mode=WAL;") after pool connection in main.rs
- **Files modified:** seeds-rs/migrations/001_initial.sql, seeds-rs/src/main.rs
- **Verification:** Server starts successfully, database created with WAL mode
- **Committed in:** 7724d95 (Task 2 commit)

**2. [Rule 3 - Blocking] Removed nested .git directory from seeds-rs/**
- **Found during:** Task 1 (commit attempt)
- **Issue:** seeds-rs/ had its own .git directory (from cargo init), preventing parent repo from tracking files inside it
- **Fix:** Removed seeds-rs/.git directory
- **Files modified:** None (deleted directory)
- **Verification:** git add succeeds for seeds-rs/ files
- **Committed in:** 2226fca (Task 1 commit)

---

**Total deviations:** 2 auto-fixed (1 bug, 1 blocking)
**Impact on plan:** Both fixes were necessary for correct operation. No scope creep.

## Issues Encountered
None beyond the auto-fixed deviations above.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Server infrastructure complete, ready for scraper module (Plan 01-02)
- DB schema and query functions ready for seed insertion
- Layout shell ready for detail page and additional routes (Plan 01-03)
- Static file serving configured for downloaded seed images

---
*Phase: 01-foundation-scraping*
*Completed: 2026-03-08*
