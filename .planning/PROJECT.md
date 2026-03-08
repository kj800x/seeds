# Seeds App

## What This Is

A single-user garden helper app for managing seed inventory and generating planting schedules. The user tracks seeds purchased from Botanical Interests across multiple years, and the app provides viability estimates and location-specific planting/transplanting dates for Halifax, MA. Built as a Rust app with Maud server-side rendering and HTMX for interactivity.

## Core Value

Take the complicated scheduling math out of starting seeds indoors — tell the app what you want to grow, and it tells you when to plant, transplant, and expect harvest.

## Requirements

### Validated

(None yet — ship to validate)

### Active

- [ ] User can add seeds to inventory by Botanical Interests product (URL or name), specifying purchase year
- [ ] App scrapes Botanical Interests product pages on-demand, capturing human-readable and machine-readable data plus images
- [ ] Scraped seed data is stored locally (SQLite) for offline access
- [ ] User can view their seed inventory organized by species/category
- [ ] App displays estimated germination viability based on species and seed age (lookup table of viability curves)
- [ ] User can select which seeds from inventory they want to grow this season
- [ ] App generates planting schedule with start-indoors and transplant-outdoors dates based on Halifax, MA frost dates (zone 6b, last frost ~May 10)
- [ ] Schedule displayed as both a visual calendar/timeline and a sorted action list
- [ ] Planting recommendations derived from scraped seed packet data (days to germination, weeks before last frost, etc.)

### Out of Scope

- Multiple users / authentication — single-user app
- Mobile app — web only
- Multiple locations — hardcoded for Halifax, MA
- Harvest tracking / garden journaling — schedule only
- Purchasing / e-commerce integration
- Pre-built full catalog scrape — on-demand only

## Context

- Existing empty Rust package at `seeds-rs/` and empty Vite app at `seeds-ui/` (Vite app will be replaced by Maud + HTMX approach)
- All seeds sourced from https://www.botanicalinterests.com/
- Seeds span multiple purchase years; viability degrades by species over time
- Halifax, MA: USDA zone 6b, average last frost ~May 10, first frost ~Oct 15
- Architecture: Rust backend serves HTML via Maud templates, HTMX for dynamic interactions, SQLite for storage
- In production, single Rust binary serves everything; in development, standard cargo watch workflow

## Constraints

- **Data source**: Botanical Interests website only — scraper must handle their page structure
- **Location**: Hardcoded Halifax, MA (zone 6b) — no location configurability needed
- **Stack**: Rust (Maud + HTMX), SQLite — no separate frontend build
- **Single user**: No auth, no multi-tenancy

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Maud + HTMX over Vite SPA | Simpler stack, most interaction is form-based, less JS to maintain | — Pending |
| On-demand scraping over pre-built catalog | Simpler, only fetch what's needed, avoids bulk scraping | — Pending |
| Species-based viability lookup table | Practical approximation, no need for experimental tracking | — Pending |
| SQLite for local storage | Single-user, local app, no need for hosted DB | — Pending |
| Hardcoded Halifax, MA | Single user in known location, simplifies v1 | — Pending |

---
*Last updated: 2026-03-08 after initialization*
