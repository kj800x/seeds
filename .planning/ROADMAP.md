# Roadmap: Seeds App

## Overview

This roadmap delivers a single-user garden seed management and planting schedule app in four phases following a strict dependency chain. The scraper and foundation come first because every downstream feature depends on having accurate scraped product data. Inventory and viability estimation follow, then schedule generation with calendar views, and finally polish features that enhance but don't define the core experience. Each phase delivers a coherent, testable capability.

## Phases

**Phase Numbering:**
- Integer phases (1, 2, 3): Planned milestone work
- Decimal phases (2.1, 2.2): Urgent insertions (marked with INSERTED)

Decimal phases appear between their surrounding integers in numeric order.

- [ ] **Phase 1: Foundation + Scraping** - Bootable server with SQLite storage and validated Botanical Interests product scraper
- [ ] **Phase 2: Seed Inventory + Viability** - Full inventory CRUD with viability estimation based on species and seed age
- [ ] **Phase 3: Season Planning + Schedule Views** - Schedule generation from inventory selections with action list and calendar views
- [ ] **Phase 4: Polish + Differentiators** - Enhanced viability indicators, focused views, and print support

## Phase Details

### Phase 1: Foundation + Scraping
**Goal**: A running Axum server that can scrape Botanical Interests product pages and persist all extracted data to SQLite
**Depends on**: Nothing (first phase)
**Requirements**: INFR-01, INFR-02, INFR-03, SCRP-01, SCRP-02, SCRP-03, SCRP-04, SCRP-05
**Success Criteria** (what must be TRUE):
  1. User can start the app and see a landing page served by the Rust binary
  2. User can paste a Botanical Interests product URL and the app extracts and displays the seed name, variety, planting instructions, days to maturity, and category
  3. Scraped seed data persists across app restarts (stored in SQLite)
  4. Product images from scraped pages are downloaded and served locally
  5. Raw HTML of every scraped page is stored for recovery if the scraper breaks
**Plans**: 3 plans

Plans:
- [x] 01-01-PLAN.md — Project setup, Axum server, SQLite schema, Maud layout shell, static files
- [ ] 01-02-PLAN.md — Scraper module (dual JSON/HTML fetch, tag parsing, image download, DB queries)
- [ ] 01-03-PLAN.md — UI pages (seed list, detail page, add form) and end-to-end verification

### Phase 2: Seed Inventory + Viability
**Goal**: Users can manage their seed collection with purchase year tracking and see how viable each seed packet remains
**Depends on**: Phase 1
**Requirements**: INVT-01, INVT-02, INVT-03, INVT-04, INVT-05, VIAB-01, VIAB-02
**Success Criteria** (what must be TRUE):
  1. User can view all seeds in their inventory showing name, variety, purchase year, and viability estimate
  2. User can click into a seed to see full scraped packet data and product images
  3. User can edit a seed entry (change purchase year, add notes) and delete seeds from inventory
  4. User can specify purchase year when adding a seed, and viability percentage updates based on species and age
**Plans**: TBD

Plans:
- [ ] 02-01: TBD
- [ ] 02-02: TBD

### Phase 3: Season Planning + Schedule Views
**Goal**: Users can select seeds to grow this season and get a complete planting schedule with start-indoors and transplant dates
**Depends on**: Phase 2
**Requirements**: PLAN-01, PLAN-02, PLAN-03, PLAN-04, PLAN-05, VIEW-01, VIEW-02
**Success Criteria** (what must be TRUE):
  1. User can mark seeds from inventory for this season's growing plan
  2. App generates start-indoors and transplant-outdoors dates derived from scraped packet data and Halifax MA frost dates
  3. Schedule correctly handles both cool-season crops (plant before last frost) and warm-season crops (plant after last frost)
  4. User can view the schedule as a sorted action list with dates and as a visual calendar/timeline of the full season
**Plans**: TBD

Plans:
- [ ] 03-01: TBD
- [ ] 03-02: TBD

### Phase 4: Polish + Differentiators
**Goal**: Enhanced viability feedback, focused schedule views, and print support that round out the user experience
**Depends on**: Phase 3
**Requirements**: VIAB-03, VIAB-04, VIAB-05, VIEW-03, VIEW-04
**Success Criteria** (what must be TRUE):
  1. Inventory list shows color-coded viability indicators (green/yellow/orange/red) at a glance
  2. App warns user about seeds nearing end of useful life and suggests over-sowing quantities for reduced germination
  3. User can view a "this week" focused view showing only current and upcoming actions
  4. User can print the planting schedule in a clean, print-friendly format
**Plans**: TBD

Plans:
- [ ] 04-01: TBD

## Progress

**Execution Order:**
Phases execute in numeric order: 1 -> 2 -> 3 -> 4

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 1. Foundation + Scraping | 2/3 | In Progress|  |
| 2. Seed Inventory + Viability | 0/? | Not started | - |
| 3. Season Planning + Schedule Views | 0/? | Not started | - |
| 4. Polish + Differentiators | 0/? | Not started | - |
