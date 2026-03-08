# Project Research Summary

**Project:** Seeds App -- Garden Seed Management & Planting Scheduler
**Domain:** Single-user local web application (Rust backend, HTML/HTMX frontend)
**Researched:** 2026-03-08
**Confidence:** MEDIUM-HIGH

## Executive Summary

This is a single-user local web application for managing garden seed inventory and generating planting schedules, built with the MASH stack (Maud, Axum, SQLite, HTMX) in Rust. The stack is well-established with multiple production examples and high-quality documentation. The core value proposition is unique: no competitor combines seed inventory management with planting scheduling, and no competitor tracks seed viability. Auto-populating seed data by scraping Botanical Interests product pages is a novel approach that eliminates tedious manual data entry.

The recommended approach is a four-phase build following the dependency graph: foundation first (server skeleton, database, views), then scraping and product storage, then seed inventory with viability, and finally schedule generation with calendar views. This order is driven by a hard dependency chain -- you cannot build inventory without scraped product data, and you cannot build schedules without inventory.

The dominant risk is the web scraper. Botanical Interests runs on Shopify, and the critical planting data (days to emerge, sow depth, weeks before frost) is NOT available in Shopify's JSON product endpoint -- it lives in rendered HTML as unstructured text. The scraper must parse free-form sentences like "Sow seeds indoors 6-8 weeks before last frost" into computable date offsets, and this parsing will vary by product category. A scraping proof-of-concept against real product pages must be the first validation milestone, before the data model or any downstream feature is finalized. Secondary risks include frost date precision (show ranges, not single dates) and viability estimation accuracy (frame as estimates with storage caveats, not guarantees).

## Key Findings

### Recommended Stack

The MASH stack is the right choice. All core crates are stable, well-maintained, and have verified integration points. Axum 0.8 + Maud 0.27 integrate via axum-core 0.5 so Maud's `Markup` type implements `IntoResponse` directly. rusqlite with bundled SQLite produces a zero-dependency binary. The entire app compiles to a single binary with embedded static assets via rust-embed.

**Core technologies:**
- **Axum 0.8**: HTTP framework -- industry standard for Rust web apps, built by the Tokio team
- **Maud 0.27**: HTML templating -- compile-time type-safe templates as Rust code, no template files
- **HTMX 2.0**: Frontend interactivity -- partial page updates without writing JavaScript, vendored locally
- **rusqlite 0.38**: Database -- synchronous SQLite with bundled binary, simplest option for single-user local app
- **reqwest 0.13 + scraper 0.25**: Web scraping -- standard Rust scraping stack, no headless browser needed
- **chrono 0.4**: Date/time -- mature date arithmetic for planting schedule calculations
- **axum-htmx 0.8**: HTMX integration -- typed extractors for HTMX headers with auto-vary support

**Version note:** chrono is MEDIUM confidence. Jiff by BurntSushi is the better long-term choice but is pre-1.0 (targeting Spring/Summer 2026). Chrono is adequate since this app only handles one fixed timezone (Halifax, MA).

### Expected Features

**Must have (table stakes):**
- Seed inventory CRUD with add-by-Botanical-Interests-URL
- Seed detail view showing scraped packet data (days to maturity, sowing depth, spacing, light)
- Purchase year tracking with germination viability estimate
- Season selection (mark which seeds to grow this year)
- Planting schedule generation from packet data + frost dates
- Action list view (what to do this week)
- Calendar/timeline view of the growing season

**Should have (differentiators):**
- Viability degradation warnings ("use this year or toss")
- Over-sow recommendations for aging seeds
- Color-coded viability indicators across inventory
- Manual seed entry for non-BI sources

**Defer (v2+):**
- Succession planting, configurable frost dates, seed quantity tracking, historical comparison
- Multi-supplier scraping, garden layout, harvest journaling, weather API (all anti-features)

**Key competitive insight:** No existing tool combines inventory + scheduling + viability tracking. Seedtime does scheduling but not inventory. Spreadsheets do inventory but not scheduling. This is the gap.

### Architecture Approach

Single Rust binary serving server-rendered HTML via Maud, with HTMX handling interactivity through partial page updates. The architecture has six clear components connected through Axum's shared state pattern. rusqlite with r2d2 connection pooling handles persistence; a shared reqwest client handles scraping. Schedule and viability engines are pure computation modules with no I/O, making them trivially testable.

**Major components:**
1. **Axum Router + Middleware** -- HTTP routing, static asset serving, request tracing
2. **Maud Views** -- HTML generation as Rust functions; dual-mode rendering (full page vs HTMX fragment)
3. **DB Layer (rusqlite + r2d2)** -- Schema migrations, CRUD operations, connection pooling via spawn_blocking
4. **Scraper Module (reqwest + scraper)** -- Fetch and parse Botanical Interests product pages, cache results
5. **Schedule Engine** -- Pure date math: frost dates + packet data = planting calendar
6. **Viability Engine** -- Species-based germination viability lookup from extension service data

**Key schema:** Three tables -- `products` (cached scraped data + raw HTML), `user_seeds` (inventory with purchase year), `season_selections` (which seeds to grow this year).

### Critical Pitfalls

1. **Planting data is NOT in Shopify's product JSON** -- The `/products/{handle}.json` endpoint only returns commerce data. All planting information lives in rendered HTML, possibly in metafields exposed via Liquid templates. Must scrape full HTML pages. Validate this before anything else.

2. **Planting data is unstructured free-form text** -- "Sow seeds indoors 6-8 weeks before last frost" must be parsed into computable offsets. Wording varies between product categories. Build regex patterns against 10-20 real product pages before finalizing the parser. Store raw text alongside parsed values for re-parsing.

3. **Scraper will break silently on Shopify theme updates** -- Botanical Interests uses Dawn theme v13.0.1. Any theme update can change HTML structure. Build validation into the scraper: if required fields come back empty, flag the scrape as broken. Always store raw HTML for recovery.

4. **Hardcoded frost dates create false precision** -- "May 10 last frost" is an average with 2-3 week variance. Display planting dates as ranges, not single dates. Make frost date user-configurable even if defaulting to Halifax, MA.

5. **Viability estimates oversimplify a complex problem** -- Storage conditions dramatically affect real viability. Frame as "estimated under ideal storage" with gradient display (excellent/good/declining/poor), not binary viable/expired. Use conservative estimates from multiple extension service sources.

## Implications for Roadmap

Based on research, suggested phase structure:

### Phase 1: Foundation + Scraping POC

**Rationale:** The scraper is the highest-risk component and the foundation of all downstream features. It must be validated before the data model is finalized. Combining it with the app skeleton means the POC runs in a real server context.
**Delivers:** Bootable Axum server with SQLite, static asset serving, and a working scraper that extracts all required fields from at least 5 Botanical Interests product pages across 3 categories (vegetable, herb, flower).
**Addresses:** BI product page scraping (table stakes), offline local data persistence
**Avoids:** Pitfall 1 (JSON endpoint lacks data), Pitfall 2 (unstructured text), Pitfall 5 (silent breakage), Pitfall 6 (scraping policy compliance)
**Stack:** Axum, Tokio, rusqlite (schema only), reqwest, scraper, tower-http, Maud (layout shell)

### Phase 2: Seed Inventory + Viability

**Rationale:** With scraped product data validated and stored, build the inventory management layer. Viability is tightly coupled to inventory (needs species + purchase year) and is low complexity.
**Delivers:** Full seed inventory CRUD (add by URL, list, detail, edit, delete), purchase year tracking, viability estimation with gradient display, scrape-and-cache flow.
**Addresses:** Seed inventory (table stakes), purchase year tracking, viability estimation, seed detail view, scrape-and-store from URL (differentiator)
**Avoids:** Pitfall 4 (viability oversimplification -- use gradient display with caveats from day one)
**Stack:** Maud views (inventory pages), rusqlite (user_seeds + products CRUD), HTMX (partial updates)

### Phase 3: Schedule Generation + Calendar

**Rationale:** Requires both product data (planting parameters) and inventory (season selection) to exist. The schedule engine is pure computation that depends on the data model established in Phases 1-2.
**Delivers:** Season selection, planting schedule engine, action list view, calendar/timeline view. Frost dates displayed as ranges.
**Addresses:** Season selection, planting schedule, action list, calendar view (all table stakes)
**Avoids:** Pitfall 3 (frost date precision -- show ranges from day one), sign error in before/after frost math (unit test both cool-season and warm-season crops)
**Stack:** chrono (date math), Maud (schedule views), HTMX (calendar interactions)

### Phase 4: Polish + Differentiators

**Rationale:** Layered on top of a working core. These are low-complexity features that enhance the user experience but are not required for the core value proposition.
**Delivers:** Viability warnings, over-sow recommendations, "this week" focused view, color-coded viability indicators, manual seed entry for non-BI sources, print-friendly schedule.
**Addresses:** All P2 features from the priority matrix
**Avoids:** Scope creep into anti-features (garden layout, journaling, multi-supplier scraping)

### Phase Ordering Rationale

- **Phase 1 before everything:** The scraper is the riskiest component and the root of the dependency graph. Every other feature depends on having accurate scraped product data. Validating scraping first prevents building on a broken foundation.
- **Inventory before Schedule:** Schedules require seed data and season selections. The inventory CRUD must exist first.
- **Viability with Inventory, not Schedule:** Viability is a property of inventory items (species + age), not schedule items. Displaying it alongside inventory entries is the natural placement.
- **Polish last:** Differentiators (warnings, recommendations, "this week" view) are low-risk, low-complexity features that layer on a working core. They do not require new architectural components.

### Research Flags

Phases likely needing deeper research during planning:
- **Phase 1 (Scraping):** HIGH priority. Must do hands-on validation of Botanical Interests page structure. The exact HTML selectors, data locations, and text patterns cannot be determined from research alone -- need to inspect real pages. This is the single biggest unknown in the project.
- **Phase 3 (Schedule):** MEDIUM priority. Cool-season vs warm-season crop handling, direct-sow-only crops, and crops with multiple planting windows need investigation during implementation. The schedule math itself is straightforward but the edge cases in planting data interpretation are numerous.

Phases with standard patterns (skip research-phase):
- **Phase 2 (Inventory):** Standard CRUD with Axum + rusqlite + Maud. Well-documented MASH stack pattern. No novel challenges.
- **Phase 4 (Polish):** All features are simple UI enhancements or computed displays on existing data. No new integrations or complex logic.

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Stack | HIGH | All crates verified with current versions and confirmed integration points. MASH stack is well-documented with multiple production examples. |
| Features | MEDIUM-HIGH | Clear competitive gap identified. Feature priorities well-defined. Viability data sourced from multiple extension services but inherently approximate. |
| Architecture | HIGH | Standard MASH stack pattern. r2d2 pooling, spawn_blocking for async bridge, fragment-based HTMX rendering all well-established. |
| Pitfalls | HIGH | Shopify scraping pitfalls well-documented. Biggest risk (planting data not in JSON) is clearly identified with mitigation strategy. |

**Overall confidence:** MEDIUM-HIGH

The stack, architecture, and pitfalls are well-understood. The primary uncertainty is whether the Botanical Interests HTML pages expose all required planting fields in a parseable format -- this can only be resolved through hands-on validation in Phase 1.

### Gaps to Address

- **Botanical Interests HTML structure:** The exact location and format of planting data on product pages has not been verified by inspecting real pages. This is the critical gap. Must be validated at the start of Phase 1 before the data model is finalized.
- **Sowing guide pages as data source:** BI has `/blogs/sowing-guides/{species}-sow-and-grow-guide` pages that may contain more structured planting data than product pages. These should be investigated as a supplementary or fallback data source during Phase 1.
- **Image handling strategy:** Whether to download and store product images locally, link to Shopify CDN, or use Shopify image transforms needs to be decided during Phase 2. For a local app, downloading thumbnails is likely best.
- **chrono vs jiff:** If jiff reaches 1.0 before Phase 3 implementation, consider switching. The date math in this app is simple enough that either library works, but jiff has a better API design.

## Sources

### Primary (HIGH confidence)
- [Axum docs (0.8.8)](https://docs.rs/axum/latest/axum/)
- [Maud docs (0.27.0)](https://docs.rs/maud/latest/maud/)
- [rusqlite docs (0.38.0)](https://docs.rs/rusqlite/latest/rusqlite/)
- [reqwest docs (0.13.2)](https://docs.rs/reqwest/latest/reqwest/)
- [axum-htmx docs (0.8.1)](https://docs.rs/axum-htmx/latest/axum_htmx/)
- [MASH Stack Architecture (Evan Schwartz)](https://emschwartz.me/building-a-fast-website-with-the-mash-stack-in-rust/)
- [Iowa State Extension - Seed Storage](https://yardandgarden.extension.iastate.edu/how-to/how-store-seeds-and-test-germination-rates)
- [University of Nebraska Extension - Seed Storage](https://go.unl.edu/seedstorage)

### Secondary (MEDIUM confidence)
- [HARM Stack (Nguyen Huy Thanh)](https://nguyenhuythanh.com/posts/the-harm-stack-considered-unharmful/)
- [Rust Web Frameworks in 2026 comparison](https://aarambhdevhub.medium.com/rust-web-frameworks-in-2026-axum-vs-actix-web-vs-rocket-vs-warp-vs-salvo-which-one-should-you-2db3792c79a2)
- [Botanical Interests - How to Read a Seed Packet](https://www.botanicalinterests.com/pages/how-to-read-a-seed-packet)
- [Shopify - Product Metafields Documentation](https://shopify.dev/docs/apps/build/custom-data)
- [High Mowing Seeds - Seed Viability Chart](https://www.highmowingseeds.com/blog/seed-viability-chart/)

### Tertiary (LOW confidence)
- [Jiff vs Chrono comparison](https://docs.rs/jiff/latest/jiff/_documentation/comparison/index.html) -- jiff pre-1.0 timeline uncertain
- Botanical Interests page HTML structure -- not directly validated, inferred from Shopify/Dawn theme patterns

---
*Research completed: 2026-03-08*
*Ready for roadmap: yes*
