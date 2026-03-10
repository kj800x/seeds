# Seeds App

## What This Is

A single-user garden seed management and planting schedule app. Users add seeds from Botanical Interests by URL, track purchases across years with viability estimates, and get location-specific planting schedules for Halifax, MA. Built as a single Rust binary with Maud server-side rendering, HTMX interactivity, and SQLite storage.

## Core Value

Take the complicated scheduling math out of starting seeds indoors — tell the app what you want to grow, and it tells you when to plant, transplant, and expect harvest.

## Requirements

### Validated

- ✓ User can add seeds by pasting a Botanical Interests product URL — v1.0
- ✓ App scrapes BI product pages extracting human-readable and machine-readable data plus images — v1.0
- ✓ All seed data stored locally in SQLite for offline access — v1.0
- ✓ User can view seed inventory with name, variety, purchase year, and viability indicator — v1.0
- ✓ App displays estimated germination viability based on species and seed age (80+ species lookup) — v1.0
- ✓ Color-coded viability indicators with end-of-life warnings and over-sowing suggestions — v1.0
- ✓ User can select seeds for this season's growing plan — v1.0
- ✓ App generates start-indoors and transplant-outdoors dates from Halifax MA frost dates and scraped packet data — v1.0
- ✓ Schedule displayed as sorted action list and visual CSS Grid timeline — v1.0
- ✓ This-week focused view and print-friendly format — v1.0

### Active

- [ ] User can add seeds from non-BI sources via manual data entry (EINV-01)
- [ ] User can track remaining seed quantity per packet (EINV-02)
- [ ] Succession planting support — multiple planting dates per crop (ESCH-01)
- [ ] Custom frost dates instead of hardcoded Halifax MA (ESCH-02)
- [ ] Historical schedules from previous seasons (ESCH-03)

### Out of Scope

- Multiple users / authentication — single-user app
- Mobile app — web only, accessible from any browser
- Multiple locations — hardcoded for Halifax, MA (single user)
- Garden bed layout planning — different product category (spatial vs temporal)
- Harvest tracking / garden journaling — splits focus from core scheduling value
- Weather API integration — static frost dates sufficient for single location
- Social / sharing features — single-user app
- Full catalog pre-scrape — on-demand only
- Multi-supplier scraping — maintenance burden, start with BI only
- E-commerce integration — out of scope

## Context

Shipped v1.0 with 3,295 lines of Rust across 4 phases.
Tech stack: Axum 0.8, Maud 0.27, HTMX 2.0, SQLite (sqlx), reqwest.
55 passing tests (viability, parser, calculator, fetcher, lookup).
Architecture: Single binary serves HTML + static assets + images.

## Constraints

- **Data source**: Botanical Interests website only — scraper handles their Shopify page structure
- **Location**: Hardcoded Halifax, MA (zone 6b) — no location configurability needed
- **Stack**: Rust (Maud + HTMX), SQLite — no separate frontend build
- **Single user**: No auth, no multi-tenancy

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Maud + HTMX over Vite SPA | Simpler stack, most interaction is form-based, less JS to maintain | ✓ Good — clean server-rendered UI with HTMX for dynamic swaps |
| On-demand scraping over pre-built catalog | Simpler, only fetch what's needed, avoids bulk scraping | ✓ Good — works well for single-user personal use |
| Species-based viability lookup table | Practical approximation, no experimental tracking needed | ✓ Good — 80+ species covers common vegetables, herbs, flowers |
| SQLite for local storage | Single-user, local app, no need for hosted DB | ✓ Good — WAL mode, fast, zero deployment |
| Hardcoded Halifax, MA | Single user in known location, simplifies v1 | ✓ Good — simplified schedule engine, configurable frost dates in v2 backlog |
| Multi-lot seed_purchases table | User feedback during Phase 2 — better than single purchase_year column | ✓ Good — supports tracking same seed across years with independent viability |
| String search over regex for planting parser | BI patterns are predictable, no regex crate needed | ✓ Good — handles 5 planting patterns reliably |
| Viability computed at render time | Depends on current year, no need to store stale values | ✓ Good — always fresh, no migration needed when year changes |
| CSS Grid timeline | Visual season overview without JS charting library | ✓ Good — lightweight, responsive, colored period bars |

---
*Last updated: 2026-03-10 after v1.0 milestone*
