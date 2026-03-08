# Architecture Patterns

**Domain:** Garden seed management / planting scheduler
**Researched:** 2026-03-08

## Recommended Architecture

Single Rust binary serving HTML via Maud templates, with HTMX for interactivity, rusqlite/r2d2 for SQLite persistence, and reqwest/scraper for on-demand data extraction from Botanical Interests.

```
                    Browser (HTMX)
                        |
                  HTTP requests
                   (HTML fragments)
                        |
                   Axum Router
                   /    |    \
              Routes  Routes  Routes
             (seeds) (schedule) (scrape)
                \      |      /
                 App State (shared)
                /              \
         r2d2 Pool          reqwest Client
        (rusqlite)          (HTTP client)
            |                    |
         SQLite          botanicalinterests.com
        (local)
```

### Component Boundaries

| Component | Responsibility | Communicates With |
|-----------|---------------|-------------------|
| **Axum Router** | HTTP request routing, middleware, static asset serving | All route handlers |
| **Route Handlers** | Request parsing, response building, coordinate between DB and views | DB layer, Maud views, Scraper |
| **Maud Views** | HTML generation as Rust functions returning `Markup` | Called by route handlers |
| **DB Layer** | All SQLite queries, schema migrations, data models | r2d2 connection pool |
| **Scraper** | Fetch and parse Botanical Interests product pages | reqwest HTTP client, DB layer (to cache results) |
| **Schedule Engine** | Planting date calculations from seed data + frost dates | DB layer (reads seed data), called by route handlers |
| **Viability Engine** | Germination viability estimates from species + seed age | Lookup table (embedded data), called by route handlers |

### Data Flow

**Adding a seed to inventory:**
```
User submits URL/name
  -> Route handler receives request
  -> Check DB: already scraped?
     YES -> Use cached product data
     NO  -> Scraper fetches page from botanicalinterests.com
         -> Parse HTML, extract structured fields
         -> Store product data in DB (products table)
  -> Create inventory entry (user_seeds table) with purchase_year
  -> Return updated inventory view (Maud -> HTML fragment)
  -> HTMX swaps fragment into page
```

**Generating planting schedule:**
```
User selects seeds for this season
  -> Route handler receives selection
  -> For each selected seed:
     -> Read product data from DB (days to germination, weeks before frost, etc.)
     -> Schedule Engine calculates:
        - Start indoors date (last_frost - weeks_before_frost)
        - Transplant date (around last_frost or after, per species)
        - Expected harvest (transplant + days_to_maturity)
     -> Viability Engine provides germination estimate for display
  -> Maud renders calendar view + action list
  -> Full page or HTMX fragment returned
```

**Viewing inventory:**
```
User navigates to inventory
  -> Route handler queries DB for all user_seeds + joined product data
  -> Viability Engine calculates current viability per seed
  -> Maud renders inventory list grouped by category
  -> Full HTML page returned
```

## Component Details

### 1. Axum Web Server

The MASH stack (Maud, Axum, SQLite, HTMX) is the established pattern for this type of Rust web app. Axum handles routing and provides extractors for clean handler signatures.

**Confidence:** HIGH (multiple production examples, well-documented pattern)

**Key patterns:**
- Axum `State` extractor shares the DB pool and HTTP client across handlers
- Maud's `Markup` implements `IntoResponse`, so handlers return HTML directly
- HTMX routes return HTML fragments; full-page routes return complete documents with layout
- A `PageLayout` extractor (from request parts) can inject common layout data

```rust
// Route organization pattern
let app = Router::new()
    .route("/", get(pages::home))
    .route("/seeds", get(seeds::list))
    .route("/seeds", post(seeds::add))
    .route("/seeds/{id}", get(seeds::detail))
    .route("/schedule", get(schedule::view))
    .route("/schedule/generate", post(schedule::generate))
    .route("/scrape", post(scrape::fetch_product))
    .with_state(app_state);
```

### 2. Maud Views (Template Layer)

Maud compiles HTML templates at build time as Rust macros. No separate template files -- views are Rust functions returning `Markup`. This gives compile-time type safety and eliminates template-related runtime errors.

**Organization:** One module per page/feature area, with shared layout and component functions.

```
src/
  views/
    mod.rs          // re-exports
    layout.rs       // base HTML shell, nav, footer
    components.rs   // reusable fragments (seed card, date badge, etc.)
    home.rs         // landing page
    seeds.rs        // inventory views (list, detail, add form)
    schedule.rs     // calendar and action list views
```

**HTMX pattern:** Each view function should be callable in two modes:
- Full page (wrapped in layout) for initial navigation
- Fragment only (no layout wrapper) for HTMX partial updates

```rust
// Pattern for dual-mode rendering
fn seed_list_fragment(seeds: &[SeedWithProduct]) -> Markup { ... }

fn seed_list_page(layout: &Layout, seeds: &[SeedWithProduct]) -> Markup {
    layout.render(seed_list_fragment(seeds))
}
```

### 3. Database Layer (rusqlite + r2d2)

Use rusqlite with r2d2 connection pooling. rusqlite is the right choice here over sqlx because:
- This is a single-user local app; async DB access adds complexity with no benefit
- rusqlite bundles SQLite into the binary (no system dependency)
- Simpler mental model -- synchronous queries in `spawn_blocking`

**Confidence:** HIGH (rusqlite is the standard for local SQLite apps in Rust)

**Schema design:**

```sql
-- Scraped product data (cached from botanicalinterests.com)
CREATE TABLE products (
    id INTEGER PRIMARY KEY,
    url TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    brand TEXT,             -- always "Botanical Interests" but store anyway
    category TEXT,          -- "Tomato", "Basil", "Pepper", etc.
    species TEXT,           -- for viability lookup
    description TEXT,
    image_url TEXT,
    -- Planting data (extracted from product page)
    days_to_germination TEXT,    -- may be a range "7-14"
    days_to_maturity TEXT,       -- may be a range "70-80"
    sowing_depth TEXT,
    plant_spacing TEXT,
    row_spacing TEXT,
    sun_requirement TEXT,        -- "Full Sun", "Part Shade", etc.
    start_indoors_weeks INTEGER, -- weeks before last frost
    direct_sow_timing TEXT,      -- relative to frost date
    plant_height TEXT,
    lifecycle TEXT,               -- "Annual", "Perennial"
    raw_html TEXT,               -- preserve original for re-parsing
    scraped_at TEXT NOT NULL     -- ISO 8601 timestamp
);

-- User's seed inventory
CREATE TABLE user_seeds (
    id INTEGER PRIMARY KEY,
    product_id INTEGER NOT NULL REFERENCES products(id),
    purchase_year INTEGER NOT NULL,
    quantity_packets INTEGER DEFAULT 1,
    notes TEXT,
    created_at TEXT NOT NULL
);

-- Season selections (which seeds to grow this year)
CREATE TABLE season_selections (
    id INTEGER PRIMARY KEY,
    user_seed_id INTEGER NOT NULL REFERENCES user_seeds(id),
    season_year INTEGER NOT NULL,
    created_at TEXT NOT NULL,
    UNIQUE(user_seed_id, season_year)
);
```

**Connection pool pattern:**

```rust
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;

struct AppState {
    db: Pool<SqliteConnectionManager>,
    http_client: reqwest::Client,
}
```

Handlers call DB operations via `spawn_blocking` to avoid blocking the async runtime:

```rust
let conn = state.db.get()?;
let seeds = tokio::task::spawn_blocking(move || {
    db::seeds::list_with_products(&conn)
}).await??;
```

### 4. Scraper Module

Use `reqwest` for HTTP fetching and `scraper` for HTML parsing. This is the standard Rust web scraping stack.

**Confidence:** HIGH (reqwest + scraper is the conventional Rust scraping stack)

**Architecture considerations:**
- Reuse a single `reqwest::Client` (shared in AppState) for connection pooling and DNS caching
- Set browser-like User-Agent headers to avoid blocking
- Store raw HTML alongside parsed fields so re-parsing is possible if the extraction logic improves
- Product pages on Botanical Interests use the URL pattern `/products/{slug}`
- The detailed planting data (days to germination, sowing depth, weeks before frost) may not be in the initial HTML -- some fields may require inspecting rendered content or specific page sections. **This is a research risk that needs validation during implementation.**
- Sowing guides at `/blogs/sowing-guides/{species}-sow-and-grow-guide` contain structured growing data and could serve as a fallback/supplementary data source

**Scraper organization:**
```
src/
  scraper/
    mod.rs          // public API: fetch_and_parse(url) -> ProductData
    client.rs       // reqwest client setup, request building
    parser.rs       // HTML parsing, CSS selector definitions
    models.rs       // ProductData struct (before DB insertion)
```

### 5. Schedule Engine

Pure computation module with no external dependencies. Takes seed planting parameters + frost dates and produces a schedule.

**Inputs:**
- Last frost date (hardcoded: May 10 for Halifax, MA)
- First frost date (hardcoded: Oct 15 for Halifax, MA)
- Per-seed: `start_indoors_weeks`, `days_to_germination`, `days_to_maturity`, `direct_sow_timing`

**Outputs:**
- Start indoors date
- Transplant outdoors date
- Direct sow date (if applicable)
- Expected harvest window
- Action list sorted by date

**Key logic:**
```
start_indoors = last_frost - (start_indoors_weeks * 7 days)
transplant    = last_frost + hardening_off_buffer (species-dependent, ~0-14 days)
harvest       = transplant + days_to_maturity
```

This module should be pure functions with no state -- easy to unit test.

### 6. Viability Engine

A lookup table mapping species to expected viable years, returning a qualitative estimate.

**Data source:** Well-established agricultural extension data. Example ranges:

| Species | Viable Years |
|---------|-------------|
| Tomatoes | 4-6 |
| Peppers | 2-5 |
| Lettuce | 2-3 |
| Cucumbers | 5-10 |
| Onions | 1 |
| Basil | 4-5 |
| Carrots | 2-3 |

**Confidence:** HIGH (agricultural extension data, well-documented)

**Implementation:** Embed the lookup table as a Rust `HashMap` or match statement. Given seed age = current_year - purchase_year, return one of:
- "Excellent" (within first half of viable range)
- "Good" (within viable range)
- "Declining" (near end of viable range)
- "Poor" (beyond viable range)

No database needed for this -- it's static reference data compiled into the binary.

## Patterns to Follow

### Pattern 1: HTMX Fragment Architecture
**What:** Every interactive element has a dedicated route returning an HTML fragment. The full page route composes these fragments into a complete page.
**When:** All user interactions that modify or filter displayed data.
**Why:** Avoids duplicating rendering logic between full-page and partial-update paths.

### Pattern 2: Shared App State via Axum Extractors
**What:** DB pool and HTTP client live in a shared `AppState` struct, injected via Axum's `State` extractor.
**When:** Every route handler that needs DB or HTTP access.
**Why:** Avoids global state, enables testing with mock state.

### Pattern 3: Scrape-and-Cache
**What:** On first request for a product URL, scrape and store. All subsequent requests use cached data.
**When:** User adds a seed by URL.
**Why:** Minimizes external requests, enables offline use after initial scrape, respects the source site.

### Pattern 4: Separation of Computation from I/O
**What:** Schedule and viability calculations are pure functions in their own modules, separate from DB and HTTP code.
**When:** Any date math or viability estimation.
**Why:** Testability, clarity, no async complexity where none is needed.

## Anti-Patterns to Avoid

### Anti-Pattern 1: Async SQLite Queries on the Main Runtime
**What:** Running rusqlite queries directly in async handlers without `spawn_blocking`.
**Why bad:** rusqlite is synchronous; blocking the tokio runtime causes performance degradation under concurrent requests.
**Instead:** Wrap DB calls in `tokio::task::spawn_blocking`.

### Anti-Pattern 2: Scraping on Every Page Load
**What:** Re-fetching product data from Botanical Interests each time a seed detail is viewed.
**Why bad:** Slow, unreliable (site could be down), disrespectful to source site.
**Instead:** Scrape once, cache in DB. Offer manual "refresh" button if data seems stale.

### Anti-Pattern 3: Mixing Schedule Logic with View Code
**What:** Calculating planting dates inside Maud template functions.
**Why bad:** Untestable, hard to debug, mixing concerns.
**Instead:** Calculate in schedule engine, pass computed dates to views.

### Anti-Pattern 4: Storing Planting Data as Unstructured Text
**What:** Keeping scraped fields like "7-14 days" as raw strings without parsing.
**Why bad:** Schedule engine needs numeric values to compute dates.
**Instead:** Parse ranges into structured types (e.g., `DayRange { min: 7, max: 14 }`) at scrape time, but also keep the raw text for display.

## Suggested Project Structure

```
seeds-rs/
  Cargo.toml
  src/
    main.rs              // Axum server setup, router, startup
    state.rs             // AppState struct (DB pool, HTTP client)
    db/
      mod.rs             // pool creation, migrations
      schema.rs          // CREATE TABLE statements, migration runner
      products.rs        // product CRUD operations
      seeds.rs           // user_seeds CRUD operations
      selections.rs      // season_selections CRUD
    views/
      mod.rs
      layout.rs          // HTML shell, nav, head, scripts
      components.rs      // reusable UI fragments
      home.rs
      seeds.rs           // inventory list, detail, add form
      schedule.rs        // calendar, action list
    routes/
      mod.rs
      home.rs
      seeds.rs           // GET/POST /seeds, GET /seeds/{id}
      schedule.rs        // GET /schedule, POST /schedule/generate
      scrape.rs          // POST /scrape (triggers on-demand scrape)
    scraper/
      mod.rs
      client.rs          // reqwest setup
      parser.rs          // HTML parsing with CSS selectors
      models.rs          // intermediate data types
    schedule/
      mod.rs             // date calculation logic
      frost_dates.rs     // Halifax, MA frost date constants
      types.rs           // PlantingEvent, Schedule types
    viability/
      mod.rs             // lookup table, viability calculation
    error.rs             // unified error type
  static/
    style.css            // minimal CSS (or Tailwind/Pico)
    htmx.min.js          // vendored HTMX (no CDN dependency)
  migrations/
    001_initial.sql
```

## Suggested Build Order (Dependencies)

Build order follows the dependency graph -- each phase builds on the previous.

```
Phase 1: Foundation
  DB layer (schema, migrations, pool setup)
  Axum server skeleton (router, state, basic routes)
  Layout/view shell (base HTML, empty pages)
  Static asset serving (CSS, HTMX JS)
  -- Validates: Rust toolchain, crate integration, app boots and serves HTML

Phase 2: Scraper + Product Storage
  reqwest client setup
  Botanical Interests page parser
  Product DB storage
  "Add seed by URL" route + form
  -- Validates: Scraping works, data extraction is accurate
  -- RISK: BI page structure may not expose all planting fields in HTML
  --        May need to iterate on parser or find supplementary data sources

Phase 3: Seed Inventory
  user_seeds CRUD
  Inventory list view
  Seed detail view (with product data)
  Viability display
  -- Depends on: Phase 2 (products must exist to add seeds)

Phase 4: Schedule Generation
  Schedule engine (pure date math)
  Season selection (pick seeds for this year)
  Calendar/timeline view
  Action list view
  -- Depends on: Phase 3 (seeds must exist to schedule)
  -- Depends on: Phase 2 (planting parameters from scraped data)
```

## Scalability Considerations

Not a major concern for a single-user local app, but noted for completeness:

| Concern | At 1 user (target) | If expanded later |
|---------|--------------------|--------------------|
| Concurrent requests | Minimal; r2d2 pool of 2-4 connections sufficient | Increase pool size |
| Scraping rate | On-demand, one at a time | Add rate limiting, queue |
| DB size | Hundreds of seeds, tiny | Still fine with SQLite for thousands |
| Seed catalog growth | Only user's seeds stored | Still fine; no full catalog |

## Sources

- [MASH Stack Architecture (Evan Schwartz)](https://emschwartz.me/building-a-fast-website-with-the-mash-stack-in-rust/) - Axum + Maud + HTMX patterns [HIGH confidence]
- [HARM Stack (Nguyen Huy Thanh)](https://nguyenhuythanh.com/posts/the-harm-stack-considered-unharmful/) - Route and state organization [HIGH confidence]
- [r2d2_sqlite crate](https://crates.io/crates/r2d2_sqlite) - Connection pool for rusqlite [HIGH confidence]
- [Axum + SQLite discussion](https://github.com/tokio-rs/axum/discussions/964) - Connection sharing pattern [HIGH confidence]
- [Rust ORMs comparison 2026](https://aarambhdevhub.medium.com/rust-orms-in-2026-diesel-vs-sqlx-vs-seaorm-vs-rusqlite-which-one-should-you-actually-use-706d0fe912f3) - rusqlite recommended for local SQLite [MEDIUM confidence]
- [Seed Viability Chart (AlboPepper)](https://albopepper.com/seed-viability-chart.php) - Species viability data [HIGH confidence]
- [Iowa State Extension - Seed Storage](https://yardandgarden.extension.iastate.edu/how-to/how-store-seeds-and-test-germination-rates) - Viability reference data [HIGH confidence]
- [Botanical Interests](https://www.botanicalinterests.com/) - Product page structure observation [MEDIUM confidence - detailed planting fields not confirmed in HTML]
