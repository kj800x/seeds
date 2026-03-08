# Phase 1: Foundation + Scraping - Context

**Gathered:** 2026-03-08
**Status:** Ready for planning

<domain>
## Phase Boundary

A running Axum server that can scrape Botanical Interests product pages and persist all extracted data to SQLite. Users can paste a product URL, see extracted seed data, and browse their collection. Infrastructure (Axum, Maud, HTMX, SQLite) and scraping are delivered together.

</domain>

<decisions>
## Implementation Decisions

### Landing page design
- Dashboard shell layout from the start — header with nav, main content area, not a minimal splash page
- Full navigation shown (Seeds, Inventory, Schedule) with future sections grayed out/disabled — user sees the full vision
- Warm and earthy visual style — muted greens/browns, organic feel, soft borders, gentle shadows
- Vanilla CSS — no build tools, CSS file served by Axum directly, custom properties for theming

### Scrape result display
- Compact list rows on main page — text-only, no thumbnails. Each row shows seed name, variety, category, days to maturity
- Click a seed row navigates to a separate detail page (/seeds/{id}) — full page, bookmarkable URL, back link
- Detail page: large hero image at top, then structured summary sections below (Growing Info, Planting, Harvest, etc.)
- Both structured summary AND collapsible original scraped text on detail page — parsed sections at top for quick reference, full original text below for completeness

### Scraping workflow
- Simple spinner/loading message ("Adding seed...") while scraping — no multi-step progress breakdown
- On success: redirect to the new seed's detail page so user immediately sees what was extracted
- On error: inline error message below the URL input ("Not a valid Botanical Interests product URL" or "Could not extract seed data"). User stays on page to retry
- Block duplicate URLs — if product was already scraped, show message "This seed is already in your collection" with link to existing entry

### Image handling
- Download all images from a product page (front of packet, back, growing photo, etc.)
- Store images in filesystem directory (e.g. data/images/{seed_id}/), served as static files by Axum
- Detail page shows single hero image (main product image) — not a gallery or grid
- List rows are text-only — no thumbnails on the main seed list

### Claude's Discretion
- Exact color palette within the warm/earthy direction
- Typography choices and spacing
- Loading spinner implementation
- Database schema design
- Scraper HTML parsing approach
- Error state styling
- How to determine "main" image vs additional images

</decisions>

<specifics>
## Specific Ideas

- Navigation should show the app's full scope even in Phase 1 — Seeds, Inventory, Schedule visible but disabled sections give a sense of where the app is headed
- Detail page should feel like reading a seed packet — image first, then the useful growing information
- The "both" approach for scraped data (structured + original) provides a safety net if the parser misses nuance from the product page

</specifics>

<code_context>
## Existing Code Insights

### Reusable Assets
- None — codebase is an empty Rust skeleton (main.rs with println) and a Vite app that will be replaced

### Established Patterns
- None — this is a greenfield phase. All patterns (routing, templates, DB access, static file serving) will be established here

### Integration Points
- seeds-rs/Cargo.toml needs dependencies: axum, maud, sqlx/rusqlite, reqwest, scraper
- seeds-ui/ directory will be replaced by Maud templates within the Rust project
- Static files (CSS, images) need an Axum static file serving route

</code_context>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 01-foundation-scraping*
*Context gathered: 2026-03-08*
