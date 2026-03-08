# Requirements: Seeds App

**Defined:** 2026-03-08
**Core Value:** Take the complicated scheduling math out of starting seeds indoors — tell the app what you want to grow, and it tells you when to plant, transplant, and expect harvest.

## v1 Requirements

### Scraping

- [ ] **SCRP-01**: User can add a seed to inventory by pasting a Botanical Interests product URL
- [ ] **SCRP-02**: App scrapes BI product page and extracts human-readable data (name, variety, description, planting instructions, days to maturity, spacing, depth)
- [ ] **SCRP-03**: App extracts machine-readable/structured data from BI product page (species, category, sun/shade, height, etc.)
- [ ] **SCRP-04**: App downloads and stores product images locally
- [ ] **SCRP-05**: App stores raw HTML of scraped page for recovery if scraper breaks

### Inventory

- [ ] **INVT-01**: User can view all seeds in their inventory with key details (name, variety, year, viability indicator)
- [ ] **INVT-02**: User can view detailed seed info page showing all scraped packet data and images
- [ ] **INVT-03**: User can specify purchase year when adding a seed
- [ ] **INVT-04**: User can edit seed inventory entries (year, notes)
- [ ] **INVT-05**: User can delete seeds from inventory

### Viability

- [ ] **VIAB-01**: App displays estimated germination viability percentage based on species and seed age
- [ ] **VIAB-02**: Viability estimation uses a species-based lookup table of viability curves
- [ ] **VIAB-03**: Inventory list shows color-coded viability indicators (green/yellow/orange/red) at a glance
- [ ] **VIAB-04**: App warns user about seeds nearing end of useful life ("use or lose" alerts)
- [ ] **VIAB-05**: App suggests over-sowing quantity for seeds with reduced germination rates

### Season Planning

- [ ] **PLAN-01**: User can select which seeds from inventory to grow this season
- [ ] **PLAN-02**: App generates planting schedule with start-indoors dates based on Halifax MA last frost (~May 10)
- [ ] **PLAN-03**: App generates transplant-outdoors dates based on frost dates and seed packet data
- [ ] **PLAN-04**: Schedule handles both cool-season crops (plant before last frost) and warm-season crops (plant after last frost) correctly
- [ ] **PLAN-05**: Planting dates derived from scraped seed packet data (weeks before last frost, days to germination, etc.)

### Schedule Views

- [ ] **VIEW-01**: User can view planting schedule as a sorted action list with dates
- [ ] **VIEW-02**: User can view planting schedule as a visual calendar/timeline showing the full season
- [ ] **VIEW-03**: User can view a "this week" focused view showing only current and upcoming week's actions
- [ ] **VIEW-04**: User can print the planting schedule in a print-friendly format

### Infrastructure

- [x] **INFR-01**: All seed data stored locally in SQLite database
- [x] **INFR-02**: App served as single Rust binary (Axum + Maud + HTMX)
- [x] **INFR-03**: App works offline after initial seed scraping

## v2 Requirements

### Extended Inventory

- **EINV-01**: User can add seeds from non-BI sources via manual data entry
- **EINV-02**: User can track remaining seed quantity per packet

### Extended Scheduling

- **ESCH-01**: User can set up succession plantings (multiple planting dates per crop)
- **ESCH-02**: User can configure custom frost dates instead of hardcoded Halifax MA
- **ESCH-03**: User can view historical schedules from previous seasons

## Out of Scope

| Feature | Reason |
|---------|--------|
| Multiple users / authentication | Single-user app, no auth needed |
| Mobile app | Web-only, accessible from any browser |
| Multiple locations | Hardcoded for Halifax, MA — single user |
| Garden bed layout planning | Different product category (spatial vs temporal); massive scope |
| Harvest tracking / garden journaling | Splits focus from core scheduling value |
| Weather API integration | Adds external dependency; static frost dates sufficient for single location |
| Social / sharing features | Single-user app |
| Full catalog pre-scrape | On-demand only; avoids bulk scraping issues |
| Multi-supplier scraping | Maintenance nightmare; start with BI only |
| E-commerce integration | Out of scope per project definition |

## Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| SCRP-01 | Phase 1 | Pending |
| SCRP-02 | Phase 1 | Pending |
| SCRP-03 | Phase 1 | Pending |
| SCRP-04 | Phase 1 | Pending |
| SCRP-05 | Phase 1 | Pending |
| INFR-01 | Phase 1 | Complete |
| INFR-02 | Phase 1 | Complete |
| INFR-03 | Phase 1 | Complete |
| INVT-01 | Phase 2 | Pending |
| INVT-02 | Phase 2 | Pending |
| INVT-03 | Phase 2 | Pending |
| INVT-04 | Phase 2 | Pending |
| INVT-05 | Phase 2 | Pending |
| VIAB-01 | Phase 2 | Pending |
| VIAB-02 | Phase 2 | Pending |
| PLAN-01 | Phase 3 | Pending |
| PLAN-02 | Phase 3 | Pending |
| PLAN-03 | Phase 3 | Pending |
| PLAN-04 | Phase 3 | Pending |
| PLAN-05 | Phase 3 | Pending |
| VIEW-01 | Phase 3 | Pending |
| VIEW-02 | Phase 3 | Pending |
| VIAB-03 | Phase 4 | Pending |
| VIAB-04 | Phase 4 | Pending |
| VIAB-05 | Phase 4 | Pending |
| VIEW-03 | Phase 4 | Pending |
| VIEW-04 | Phase 4 | Pending |

**Coverage:**
- v1 requirements: 27 total
- Mapped to phases: 27
- Unmapped: 0

---
*Requirements defined: 2026-03-08*
*Last updated: 2026-03-08 after roadmap creation*
