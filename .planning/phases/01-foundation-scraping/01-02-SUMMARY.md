---
phase: 01-foundation-scraping
plan: 02
subsystem: scraper
tags: [reqwest, scraper, shopify-json-api, html-parsing, image-download, serde]

# Dependency graph
requires:
  - phase: 01-01
    provides: "Axum server, SQLite schema, AppState, DB query functions, AppError type"
provides:
  - Scraper module with dual-fetch strategy (JSON API + HTML)
  - Tag parser extracting category, subcategory, light, frost tolerance, organic/heirloom
  - HTML parser for growing details (best-effort with graceful fallback)
  - Image downloader with partial failure handling
  - scrape_and_save orchestrator with duplicate detection
  - DuplicateSeed error variant for blocking duplicate URLs
affects: [01-03]

# Tech tracking
tech-stack:
  added: []
  patterns: [dual-fetch scraping (JSON API + HTML), tag convention parsing, best-effort HTML extraction with raw storage fallback, partial-success image downloading]

key-files:
  created:
    - seeds-rs/src/scraper/mod.rs
    - seeds-rs/src/scraper/fetcher.rs
    - seeds-rs/src/scraper/parser.rs
    - seeds-rs/src/scraper/images.rs
  modified:
    - seeds-rs/src/error.rs
    - seeds-rs/src/main.rs

key-decisions:
  - "HTML growing detail extraction uses multi-strategy best-effort approach (metafield selectors, product description blocks, tab panels) with graceful None fallback since exact BI HTML structure is unknown"
  - "DuplicateSeed is a distinct AppError variant (not ScraperError) returning HTTP 409 Conflict for clear UX handling"
  - "fetch_product returns handle alongside product data so orchestrator can use it for duplicate checking and canonical URL construction"

patterns-established:
  - "Scraper module pattern: fetcher (HTTP), parser (extraction), images (download), mod.rs (orchestrator)"
  - "Partial success pattern: image downloads and image DB inserts log warnings but continue on individual failures"
  - "URL normalization: extract handle from URL, lowercase, strip query params/trailing slash, use as unique key"

requirements-completed: [SCRP-02, SCRP-03, SCRP-04, SCRP-05]

# Metrics
duration: 3min
completed: 2026-03-08
---

# Phase 1 Plan 2: Scraper Module Summary

**Dual-fetch scraper with Shopify JSON API for structured data, HTML parsing for growing details, image download to filesystem, and raw HTML storage for recovery**

## Performance

- **Duration:** 3 min
- **Started:** 2026-03-08T21:47:50Z
- **Completed:** 2026-03-08T21:51:30Z
- **Tasks:** 2 (1 implemented, 1 already complete from Plan 01)
- **Files modified:** 6

## Accomplishments
- Scraper module with clear separation: fetcher (HTTP), parser (extraction), images (download), mod (orchestration)
- Dual-fetch strategy: JSON API for structured product data, HTML page for growing details and raw storage
- Tag parser extracts category, subcategory, light requirement, frost tolerance, organic/heirloom from comma-separated tags
- HTML growing details parser with multi-strategy extraction (metafield selectors, description blocks, tab panels) and graceful None fallback
- Image downloader with content-type detection, partial failure handling, and filesystem storage at data/images/{seed_id}/
- scrape_and_save orchestrator wiring everything together with duplicate detection via product handle
- 11 unit tests for URL handle extraction, tag parsing, and utility functions

## Task Commits

Each task was committed atomically:

1. **Task 1: Scraper fetcher, parser, and image downloader** - `d6a983f` (feat)
2. **Task 2: Fully implement DB query functions** - Already complete from Plan 01, no commit needed

## Files Created/Modified
- `seeds-rs/src/scraper/mod.rs` - Module entry + scrape_and_save orchestrator function
- `seeds-rs/src/scraper/fetcher.rs` - Shopify JSON API + HTML page fetching, URL handle extraction
- `seeds-rs/src/scraper/parser.rs` - Tag parsing (ParsedTags) and HTML growing detail extraction (GrowingDetails)
- `seeds-rs/src/scraper/images.rs` - Image downloading to filesystem with content-type detection
- `seeds-rs/src/error.rs` - Added DuplicateSeed variant with existing_id
- `seeds-rs/src/main.rs` - Registered scraper module

## Decisions Made
- HTML growing detail extraction uses a multi-strategy approach (metafield selectors, product description blocks, tab/accordion panels) since exact BI HTML structure is unknown (LOW confidence from research). Raw HTML is stored (SCRP-05) so parsing can be refined later.
- DuplicateSeed is a distinct AppError variant returning HTTP 409 Conflict, separate from ScraperError, for clearer UX handling in Plan 03.
- fetch_product returns the extracted handle alongside product data, avoiding redundant URL parsing in the orchestrator.
- Task 2 (DB queries) was already fully implemented in Plan 01 -- no changes needed. Plan 01's executor completed all query functions including NewSeed/NewSeedImage structs.

## Deviations from Plan

None - plan executed exactly as written. Task 2 required no work since Plan 01 already delivered complete DB query implementations.

## Issues Encountered
None.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Scraper module complete, ready for route wiring in Plan 01-03
- scrape_and_save function ready to be called from POST /seeds/add handler
- DuplicateSeed error ready for UX handling (link to existing seed)
- Image files stored at paths compatible with existing ServeDir at /images
- All "never used" warnings are expected -- functions will be called once routes are wired in Plan 03

---
*Phase: 01-foundation-scraping*
*Completed: 2026-03-08*
