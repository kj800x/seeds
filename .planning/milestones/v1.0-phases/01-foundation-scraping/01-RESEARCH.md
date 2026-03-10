# Phase 1: Foundation + Scraping - Research

**Researched:** 2026-03-08
**Domain:** Rust web server (Axum), HTML scraping (scraper + reqwest), SQLite persistence (sqlx), server-side HTML templating (Maud + HTMX)
**Confidence:** MEDIUM-HIGH

## Summary

This phase builds a greenfield Rust web application using the "MASH stack" (Maud, Axum, SQLx, HTMX) that scrapes Botanical Interests product pages and persists seed data to SQLite. The existing codebase is an empty Rust skeleton (`edition = "2024"`, no dependencies) and a Vite/React UI that will be replaced entirely.

A critical discovery: Botanical Interests runs on Shopify and exposes a public `.json` API endpoint (`/products/{handle}.json`) that provides structured product data (name, description, images, tags, price). However, detailed growing information (days to maturity, sow depth, spacing, germination) is stored as Shopify metafields rendered server-side into the HTML by the Liquid theme -- these fields are NOT available via the JSON API. The scraper must therefore use a dual-fetch strategy: the JSON endpoint for clean structured data AND raw HTML parsing for growing details. The raw HTML also satisfies the SCRP-05 recovery requirement.

The tags field in the JSON API contains valuable structured data including light requirements ("Full Sun to Part Shade"), categories ("Cat - Vegetables", "SubCat - Tomato"), and growing characteristics ("Frost Tolerant", "Good for Containers").

**Primary recommendation:** Use Axum 0.8 + Maud + HTMX for the server-rendered UI, sqlx 0.8 with SQLite for persistence, and reqwest + scraper for the dual JSON/HTML scraping approach. Serve static files (CSS, downloaded images) via tower-http ServeDir.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- Dashboard shell layout from the start -- header with nav, main content area, not a minimal splash page
- Full navigation shown (Seeds, Inventory, Schedule) with future sections grayed out/disabled -- user sees the full vision
- Warm and earthy visual style -- muted greens/browns, organic feel, soft borders, gentle shadows
- Vanilla CSS -- no build tools, CSS file served by Axum directly, custom properties for theming
- Compact list rows on main page -- text-only, no thumbnails. Each row shows seed name, variety, category, days to maturity
- Click a seed row navigates to a separate detail page (/seeds/{id}) -- full page, bookmarkable URL, back link
- Detail page: large hero image at top, then structured summary sections below (Growing Info, Planting, Harvest, etc.)
- Both structured summary AND collapsible original scraped text on detail page -- parsed sections at top for quick reference, full original text below for completeness
- Simple spinner/loading message ("Adding seed...") while scraping -- no multi-step progress breakdown
- On success: redirect to the new seed's detail page so user immediately sees what was extracted
- On error: inline error message below the URL input ("Not a valid Botanical Interests product URL" or "Could not extract seed data"). User stays on page to retry
- Block duplicate URLs -- if product was already scraped, show message "This seed is already in your collection" with link to existing entry
- Download all images from a product page (front of packet, back, growing photo, etc.)
- Store images in filesystem directory (e.g. data/images/{seed_id}/), served as static files by Axum
- Detail page shows single hero image (main product image) -- not a gallery or grid
- List rows are text-only -- no thumbnails on the main seed list

### Claude's Discretion
- Exact color palette within the warm/earthy direction
- Typography choices and spacing
- Loading spinner implementation
- Database schema design
- Scraper HTML parsing approach
- Error state styling
- How to determine "main" image vs additional images

### Deferred Ideas (OUT OF SCOPE)
None -- discussion stayed within phase scope
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| INFR-01 | All seed data stored locally in SQLite database | sqlx 0.8 with SQLite feature; schema design for seeds, images, raw_html |
| INFR-02 | App served as single Rust binary (Axum + Maud + HTMX) | Axum 0.8 + Maud 0.27 + HTMX 2.0 CDN; tower-http for static files |
| INFR-03 | App works offline after initial seed scraping | All data in SQLite + local filesystem images; HTMX from local file not CDN |
| SCRP-01 | User can add a seed by pasting a Botanical Interests product URL | URL validation, form handling, duplicate detection via stored product handle |
| SCRP-02 | App scrapes BI product page and extracts human-readable data | Dual-fetch: JSON API for name/description + HTML parsing for growing details |
| SCRP-03 | App extracts machine-readable/structured data from BI product page | Tags from JSON API contain category, subcategory, light requirements, growing traits |
| SCRP-04 | App downloads and stores product images locally | Image URLs from JSON API images array; download with reqwest, store in data/images/{seed_id}/ |
| SCRP-05 | App stores raw HTML of scraped page for recovery | Store full HTML response body in database or filesystem |
</phase_requirements>

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| axum | 0.8 | HTTP framework, routing, request handling | Tokio ecosystem standard; 0.8 uses `/{param}` syntax |
| maud | 0.27 | Compile-time HTML templating | Zero-cost abstractions, type-safe, has `axum` feature flag |
| sqlx | 0.8 | Async SQLite database access | Compile-time query checking, async, pure Rust |
| reqwest | 0.13 | HTTP client for scraping | De facto Rust HTTP client, async, handles redirects |
| scraper | latest | HTML parsing with CSS selectors | Built on html5ever (Servo's parser), CSS selector API |
| tokio | 1 | Async runtime | Required by axum, sqlx, reqwest |
| tower-http | 0.6 | Static file serving middleware | `ServeDir` for CSS and image files |
| serde + serde_json | 1 | JSON deserialization | Parsing Shopify JSON API responses |
| htmx | 2.0 | Client-side interactivity (JS library) | Served as local static file for offline support |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| tower | 0.5 | Middleware composition | If custom middleware needed (logging, etc.) |
| tracing + tracing-subscriber | latest | Structured logging | Debugging scraper issues, request logging |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| sqlx | rusqlite | rusqlite is sync-only but simpler; sqlx is async and has migration support built-in |
| scraper | select.rs | scraper has broader adoption and html5ever backing |
| Maud | Askama | Askama uses file-based templates (more familiar); Maud is inline macros (faster compilation, type-safe) |

**Installation (Cargo.toml dependencies):**
```toml
[dependencies]
axum = "0.8"
maud = { version = "0.27", features = ["axum"] }
sqlx = { version = "0.8", features = ["runtime-tokio", "sqlite"] }
reqwest = { version = "0.13", features = ["json"] }
scraper = "0.22"
tokio = { version = "1", features = ["full"] }
tower-http = { version = "0.6", features = ["fs"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tracing = "0.1"
tracing-subscriber = "0.3"
```

## Architecture Patterns

### Recommended Project Structure
```
seeds-rs/
├── Cargo.toml
├── src/
│   ├── main.rs              # Server startup, router assembly
│   ├── routes/
│   │   ├── mod.rs
│   │   ├── home.rs           # Landing page / seed list
│   │   ├── seeds.rs          # Seed detail, add seed form/handler
│   │   └── static_files.rs   # CSS, images, htmx.js serving
│   ├── templates/
│   │   ├── mod.rs
│   │   ├── layout.rs         # Shell layout (header, nav, footer)
│   │   ├── home.rs           # Seed list template
│   │   └── seed_detail.rs    # Individual seed page template
│   ├── scraper/
│   │   ├── mod.rs
│   │   ├── fetcher.rs        # HTTP fetching (JSON API + HTML)
│   │   ├── parser.rs         # HTML parsing for growing details
│   │   └── images.rs         # Image downloading
│   ├── db/
│   │   ├── mod.rs
│   │   ├── models.rs         # Seed struct, Image struct
│   │   └── queries.rs        # Insert/select/check duplicate
│   └── error.rs              # App error types
├── migrations/
│   └── 001_initial.sql       # SQLite schema
├── static/
│   ├── style.css             # Vanilla CSS with custom properties
│   └── htmx.min.js           # Local copy for offline support
└── data/
    └── images/               # Downloaded seed images (runtime)
```

### Pattern 1: Axum Router with Shared State
**What:** Pass a database pool as shared state to all handlers
**When to use:** Every route that needs DB access
**Example:**
```rust
use axum::{Router, routing::get, extract::State};
use sqlx::SqlitePool;

#[derive(Clone)]
struct AppState {
    db: SqlitePool,
    data_dir: std::path::PathBuf,
}

#[tokio::main]
async fn main() {
    let pool = SqlitePool::connect("sqlite:data/seeds.db?mode=rwc")
        .await.unwrap();
    sqlx::migrate!().run(&pool).await.unwrap();

    let state = AppState {
        db: pool,
        data_dir: std::path::PathBuf::from("data"),
    };

    let app = Router::new()
        .route("/", get(home))
        .route("/seeds/{id}", get(seed_detail))
        .route("/seeds/add", get(add_seed_form).post(add_seed))
        .nest_service("/static", ServeDir::new("static"))
        .nest_service("/images", ServeDir::new("data/images"))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
```

### Pattern 2: Maud Layout Composition
**What:** Reusable page shell via function composition (not template inheritance)
**When to use:** Every page needs the same header/nav/footer
**Example:**
```rust
use maud::{html, Markup, DOCTYPE, PreEscaped};

fn layout(title: &str, content: Markup) -> Markup {
    html! {
        (DOCTYPE)
        html lang="en" {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
                title { (title) " | Seeds" }
                link rel="stylesheet" href="/static/style.css";
                script src="/static/htmx.min.js" {}
            }
            body {
                header.app-header {
                    h1.logo { "Seeds" }
                    nav.main-nav {
                        a.nav-link.active href="/" { "Seeds" }
                        span.nav-link.disabled { "Inventory" }
                        span.nav-link.disabled { "Schedule" }
                    }
                }
                main.content {
                    (content)
                }
            }
        }
    }
}
```

### Pattern 3: HTMX Form Submission with Loading State
**What:** Use HTMX to POST the scrape URL and show a spinner, then redirect
**When to use:** The "add seed" form
**Example:**
```rust
fn add_seed_form() -> Markup {
    html! {
        section.add-seed {
            h2 { "Add a Seed" }
            form hx-post="/seeds/add"
                 hx-indicator=".spinner"
                 hx-target=".form-result" {
                input type="url" name="url"
                      placeholder="Paste a Botanical Interests product URL..."
                      required;
                button type="submit" { "Add Seed" }
                span.spinner { "Adding seed..." }
            }
            div.form-result {}
        }
    }
}
```

### Pattern 4: Dual-Fetch Scraping Strategy
**What:** Fetch both the Shopify JSON API and the HTML page for each product
**When to use:** Every scrape operation
**Example:**
```rust
// Shopify JSON API gives us structured data
let json_url = format!("{}.json", product_url);
let json_resp: ShopifyProduct = reqwest::get(&json_url)
    .await?.json().await?;

// HTML page gives us growing details rendered from metafields
let html_resp = reqwest::get(&product_url).await?.text().await?;
let document = scraper::Html::parse_document(&html_resp);

// Store raw HTML for recovery (SCRP-05)
save_raw_html(&db, seed_id, &html_resp).await?;
```

### Anti-Patterns to Avoid
- **Spawning blocking tasks for DB access:** sqlx is already async; don't wrap in `spawn_blocking`
- **Using HTMX CDN:** Must serve htmx.min.js locally for offline support (INFR-03)
- **Parsing HTML for data available in JSON:** Use the `.json` endpoint for name, description, images, tags -- only parse HTML for growing details
- **Storing images in the database:** Use filesystem storage with DB references; large blobs in SQLite hurt performance

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| HTML parsing | Custom regex parser | `scraper` crate with CSS selectors | HTML is messy; regex can't handle malformed markup |
| HTTP client | Raw TCP/hyper | `reqwest` | Handles redirects, TLS, cookies, timeouts |
| Database migrations | Manual CREATE TABLE scripts | `sqlx::migrate!()` macro | Tracks applied migrations, runs at startup |
| Static file serving | Custom file-read handlers | `tower_http::services::ServeDir` | Handles MIME types, caching headers, 404s |
| URL validation | Regex URL matching | `url` crate or string prefix check | BI URLs follow `/products/{handle}` pattern; simple prefix check sufficient |

**Key insight:** The Shopify JSON API eliminates the need to scrape basic product info from HTML. Only growing details (metafield-rendered content) require HTML parsing.

## Common Pitfalls

### Pitfall 1: Axum 0.8 Path Syntax
**What goes wrong:** Using old `/:id` path syntax from 0.7 examples
**Why it happens:** Most tutorials and Stack Overflow answers show 0.7 syntax
**How to avoid:** Always use `/{id}` and `/{*rest}` syntax in axum 0.8
**Warning signs:** Compile error about path parameters

### Pitfall 2: SQLite WAL Mode
**What goes wrong:** Concurrent reads block on writes with default journal mode
**Why it happens:** SQLite defaults to DELETE journal mode
**How to avoid:** Enable WAL mode on connection: `PRAGMA journal_mode=WAL;`
**Warning signs:** "database is locked" errors under concurrent access

### Pitfall 3: Missing Shopify JSON Fields
**What goes wrong:** Assuming all products have the same JSON structure
**Why it happens:** Some products may lack images, have different variant structures, or missing tags
**How to avoid:** Use `Option<T>` for all non-guaranteed fields in the deserialization struct
**Warning signs:** Deserialization panics on certain products

### Pitfall 4: Image Download Failures
**What goes wrong:** Scraper crashes if image download fails partway through
**Why it happens:** Network errors, CDN issues, large files
**How to avoid:** Download images individually with error handling; partial success is acceptable (save seed data even if some images fail)
**Warning signs:** Entire scrape operation fails due to one bad image URL

### Pitfall 5: HTMX Offline Requirement
**What goes wrong:** Loading HTMX from CDN means app breaks offline
**Why it happens:** Default tutorials show CDN loading
**How to avoid:** Download htmx.min.js (2.0.8) and serve from `/static/htmx.min.js`
**Warning signs:** App stops working without internet after initial setup

### Pitfall 6: BI Product URL Normalization
**What goes wrong:** Duplicate seeds stored because URL variations aren't normalized
**Why it happens:** Users might paste URLs with query parameters, trailing slashes, or different casing
**How to avoid:** Extract the product handle from the URL and use it as the unique key; normalize by stripping query params and lowercasing
**Warning signs:** Same seed appears multiple times in the list

### Pitfall 7: Growing Details May Be Absent
**What goes wrong:** Scraper expects growing details in HTML but some products don't have them
**Why it happens:** Not all BI products have complete metafield data; the Shopify theme only renders what exists
**How to avoid:** All growing detail fields should be `Option<String>` in the schema; the scraper should gracefully handle missing sections
**Warning signs:** Scraper errors on products with minimal information

## Code Examples

### Shopify JSON API Response Structure
```rust
// Source: Verified against live BI .json endpoint (2026-03-08)
#[derive(Debug, Deserialize)]
struct ShopifyProductResponse {
    product: ShopifyProduct,
}

#[derive(Debug, Deserialize)]
struct ShopifyProduct {
    id: i64,
    title: String,
    body_html: Option<String>,
    handle: String,
    tags: String,  // Comma-separated: "Cat - Vegetables, SubCat - Tomato, Full Sun to Part Shade"
    images: Vec<ShopifyImage>,
    image: Option<ShopifyImage>,  // Primary image
}

#[derive(Debug, Deserialize)]
struct ShopifyImage {
    id: i64,
    src: String,
    position: i32,
    width: Option<i32>,
    height: Option<i32>,
    alt: Option<String>,
}
```

### Tag Parsing for Structured Data (SCRP-03)
```rust
// Tags from JSON API contain structured data as conventions:
// "Cat - Vegetables" -> category = "Vegetables"
// "SubCat - Tomato" -> subcategory = "Tomato"
// "Full Sun to Part Shade" -> light requirement
// "Frost Tolerant" -> frost tolerance
// "Good for Containers" -> container suitable
// "heirloom-vegetables" -> heirloom = true
// "organic" -> organic = true

fn parse_tags(tags_str: &str) -> ParsedTags {
    let tags: Vec<&str> = tags_str.split(", ").collect();
    let mut result = ParsedTags::default();
    for tag in &tags {
        if let Some(cat) = tag.strip_prefix("Cat - ") {
            result.category = Some(cat.to_string());
        } else if let Some(sub) = tag.strip_prefix("SubCat - ") {
            result.subcategory = Some(sub.to_string());
        }
        // ... other patterns
    }
    result
}
```

### Suggested SQLite Schema
```sql
-- migrations/001_initial.sql
CREATE TABLE IF NOT EXISTS seeds (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    product_handle TEXT NOT NULL UNIQUE,  -- "sun-gold-cherry-tomato-seeds"
    source_url TEXT NOT NULL,
    title TEXT NOT NULL,
    description TEXT,                     -- body_html from JSON API
    category TEXT,                        -- parsed from tags: "Cat - Vegetables"
    subcategory TEXT,                     -- parsed from tags: "SubCat - Tomato"
    light_requirement TEXT,              -- parsed from tags
    frost_tolerance TEXT,                -- parsed from tags
    is_organic BOOLEAN DEFAULT FALSE,
    is_heirloom BOOLEAN DEFAULT FALSE,

    -- Growing details (from HTML scraping, all optional)
    days_to_maturity TEXT,
    sow_depth TEXT,
    plant_spacing TEXT,
    germination_info TEXT,
    planting_instructions TEXT,          -- Full text block of planting info
    growing_instructions TEXT,           -- Full text block of growing info
    harvest_instructions TEXT,           -- Full text block of harvest info

    -- Raw storage
    raw_html TEXT,                        -- Full HTML page (SCRP-05)
    shopify_product_id INTEGER,
    tags_raw TEXT,                        -- Original comma-separated tags

    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS seed_images (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    seed_id INTEGER NOT NULL REFERENCES seeds(id) ON DELETE CASCADE,
    shopify_image_id INTEGER,
    position INTEGER NOT NULL,           -- 1 = primary/hero image
    original_url TEXT NOT NULL,
    local_filename TEXT NOT NULL,        -- e.g., "1.jpg"
    width INTEGER,
    height INTEGER,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);
```

### Static File Serving
```rust
// Source: axum static-file-server example + tower-http docs
use tower_http::services::ServeDir;

let app = Router::new()
    // ... routes ...
    .nest_service("/static", ServeDir::new("static"))
    .nest_service("/images", ServeDir::new("data/images"));
```

### Image Download Pattern
```rust
async fn download_images(
    client: &reqwest::Client,
    images: &[ShopifyImage],
    seed_id: i64,
    data_dir: &Path,
) -> Vec<LocalImage> {
    let dir = data_dir.join("images").join(seed_id.to_string());
    tokio::fs::create_dir_all(&dir).await.unwrap();

    let mut local_images = Vec::new();
    for (i, img) in images.iter().enumerate() {
        let filename = format!("{}.jpg", i + 1);
        let path = dir.join(&filename);

        match client.get(&img.src).send().await {
            Ok(resp) => {
                if let Ok(bytes) = resp.bytes().await {
                    if tokio::fs::write(&path, &bytes).await.is_ok() {
                        local_images.push(LocalImage {
                            position: img.position,
                            filename,
                        });
                    }
                }
            }
            Err(e) => {
                tracing::warn!("Failed to download image {}: {}", img.src, e);
                // Continue -- don't fail the whole scrape for one image
            }
        }
    }
    local_images
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| axum 0.7 `/:param` | axum 0.8 `/{param}` | Jan 2025 | All route definitions use new syntax |
| `#[async_trait]` on extractors | Native async trait methods | axum 0.8 / Rust 1.75+ | Simpler custom extractor code |
| rusqlite (sync) | sqlx (async) | Ongoing | Better fit with tokio/axum async model |
| HTMX 1.x | HTMX 2.0 | Mid 2024 | Minor API changes; dropped IE support |
| Rust edition 2021 | Rust edition 2024 | Rust 1.85, Feb 2025 | Project already uses 2024 edition |

**Deprecated/outdated:**
- axum 0.7 path syntax (`/:id`) -- must use `/{id}` in 0.8
- `#[async_trait]` macro for axum extractors -- no longer needed

## Botanical Interests Scraping Details

### URL Format
```
https://www.botanicalinterests.com/products/{product-handle}
```
Product handles are lowercase-hyphenated: `sun-gold-cherry-tomato-seeds`, `italian-genovese-basil-seeds`

### JSON API Endpoint
```
https://www.botanicalinterests.com/products/{product-handle}.json
```
Returns a `ShopifyProductResponse` with structured data. No authentication required.

### What's Available via JSON API (HIGH confidence)
- Product title, description (body_html), handle
- All images with URLs, dimensions, positions
- Tags (comma-separated string with category conventions)
- Price, SKU, vendor, product type
- Shopify product ID

### What's NOT in JSON API -- Requires HTML Parsing (MEDIUM confidence)
- Days to maturity
- Sow depth and spacing
- Germination temperature/time
- When to sow (indoor start, direct sow dates relative to frost)
- Plant height
- Harvest instructions
- Botanical/species name

**Important note:** The growing details are rendered server-side from Shopify metafields into the HTML page by the Liquid theme. The WebFetch tool could not reliably extract this content during research, which suggests the content may be in dynamically loaded sections or complex DOM structures. The scraper implementation will need to investigate the actual HTML structure by fetching real pages and inspecting the DOM. This is the highest-risk component of the phase.

### Recommended "Main Image" Heuristic
The JSON API returns images with `position` values. The image at `position: 1` is consistently the front-of-packet product image. Use `position == 1` to determine the hero image.

## Open Questions

1. **HTML Structure for Growing Details**
   - What we know: Growing data exists on BI product pages (Google indexes it), stored as Shopify metafields rendered via Liquid templates
   - What's unclear: Exact CSS selectors/DOM structure needed to extract growing details. WebFetch couldn't capture this content, likely because it's in dynamically assembled Shopify sections
   - Recommendation: First implementation task should include fetching a real page with reqwest and inspecting the raw HTML to identify CSS selectors. Build the parser iteratively. Store raw HTML so parsing can be refined later (SCRP-05 provides this safety net)

2. **Shopify Rate Limiting**
   - What we know: Public Shopify storefront has rate limits, but single-user adding seeds one at a time is unlikely to hit them
   - What's unclear: Exact rate limit thresholds for the JSON API
   - Recommendation: Add a small delay between JSON and HTML fetches; include a User-Agent header. Not a real concern for this use case

3. **Image File Extensions**
   - What we know: BI images are served from Shopify CDN as `.jpg` files
   - What's unclear: Whether some products use `.png` or `.webp`
   - Recommendation: Detect content-type from response headers and use appropriate extension

## Sources

### Primary (HIGH confidence)
- Shopify JSON API endpoint -- verified by fetching live data from `botanicalinterests.com/products/{handle}.json` (2026-03-08)
- [Axum 0.8.0 announcement](https://tokio.rs/blog/2025-01-01-announcing-axum-0-8-0) -- path syntax changes, async trait support
- [Maud official docs](https://maud.lambda.xyz/) -- layout composition, axum feature flag
- [sqlx on crates.io](https://crates.io/crates/sqlx) -- v0.8.6, SQLite support
- [axum static file server example](https://github.com/tokio-rs/axum/blob/main/examples/static-file-server/src/main.rs)

### Secondary (MEDIUM confidence)
- [MASH stack article](https://emschwartz.me/building-a-fast-website-with-the-mash-stack-in-rust/) -- architectural patterns for Maud+Axum+SQLx+HTMX
- [reqwest on crates.io](https://crates.io/crates/reqwest) -- v0.13.2
- [HTMX 2.0](https://htmx.org/) -- v2.0.8 current

### Tertiary (LOW confidence)
- BI HTML structure for growing details -- could not verify exact selectors; requires hands-on investigation during implementation

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH -- all crates are well-established, versions verified on crates.io
- Architecture: HIGH -- MASH stack is a documented pattern with real examples
- Scraping (JSON API): HIGH -- verified against live endpoint, structure documented
- Scraping (HTML growing details): LOW -- could not extract actual HTML structure; highest-risk area
- Pitfalls: MEDIUM -- based on known Shopify/axum patterns and general web scraping experience

**Research date:** 2026-03-08
**Valid until:** 2026-04-08 (stable ecosystem, 30 days)
