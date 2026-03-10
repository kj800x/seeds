# Project Retrospective

*A living document updated after each milestone. Lessons feed forward into future planning.*

## Milestone: v1.0 — Seeds App MVP

**Shipped:** 2026-03-10
**Phases:** 4 | **Plans:** 8

### What Was Built
- Full-stack seed management app (Axum + Maud + HTMX + SQLite)
- Botanical Interests product scraper with dual JSON/HTML fetch and image downloading
- Multi-lot purchase tracking with species-based viability estimation (80+ species)
- Season planning engine with frost-relative date calculation from scraped packet data
- Schedule views: sorted action list + CSS Grid timeline + this-week focus + print stylesheet
- Viability polish: color-coded indicators, end-of-life warnings, over-sowing suggestions

### What Worked
- Scraper-first dependency chain: building the data layer first meant every downstream feature had real data to work with
- Maud + HTMX approach: server-rendered HTML with targeted HTMX swaps kept the UI simple with zero JS framework overhead
- Multi-lot seed_purchases deviation: user feedback during Phase 2 improved the data model early, avoiding costly retrofit later
- String search over regex for planting parser: kept it simple, handles all 5 BI patterns without external regex crate
- Research-first phase workflow: domain research before planning caught key patterns (Shopify JSON API, planting text formats)

### What Was Inefficient
- VIEW-03 and VIEW-04 not tracked in SUMMARY frontmatter or traceability table — documentation staleness caught only at audit time
- Phase 4 PLAN frontmatter omitted VIEW-03/VIEW-04 from requirements field despite implementing them — traceability gap
- Dead-code warnings accumulated for deserialization struct fields (minor but noisy)

### Patterns Established
- Compute viability at render time (not stored) since it depends on current year
- Layout helper `layout_with_nav(title, active_nav, content)` for consistent nav highlighting
- HTMX pattern: `hx-swap="outerHTML"` on buttons for toggle state without page reload
- `stopPropagation` on nested clickable elements inside list item links
- Migration chain preserves data: migration 003 auto-migrated purchase_year to seed_purchases table

### Key Lessons
1. Track all implemented requirements in SUMMARY frontmatter immediately — don't defer documentation updates
2. Dual-fetch strategy (JSON API + HTML) is more robust than HTML-only scraping for Shopify sites
3. Best-effort extraction with graceful None fallback works well for unstructured HTML data
4. CSS Grid timelines are lightweight alternatives to JS charting libraries for date visualization

### Cost Observations
- Model mix: balanced profile (sonnet for most agents, opus for orchestration)
- Total execution time: ~41 minutes across 8 plans
- Notable: entire v1.0 MVP shipped in a single day session

---

## Cross-Milestone Trends

### Process Evolution

| Milestone | Phases | Plans | Key Change |
|-----------|--------|-------|------------|
| v1.0 | 4 | 8 | Initial project — established scraper-first dependency chain |

### Cumulative Quality

| Milestone | Tests | LOC | Files |
|-----------|-------|-----|-------|
| v1.0 | 55 | 3,295 Rust | 66 |

### Top Lessons (Verified Across Milestones)

1. (First milestone — lessons pending cross-validation in future milestones)
