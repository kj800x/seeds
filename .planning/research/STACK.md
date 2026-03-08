# Technology Stack

**Project:** Seeds App -- Garden Seed Management & Planting Scheduler
**Researched:** 2026-03-08

## Recommended Stack

### Core Framework

| Technology | Version | Purpose | Why | Confidence |
|------------|---------|---------|-----|------------|
| Axum | 0.8.x | HTTP framework | Industry standard for new Rust web apps in 2025/2026. Built by the Tokio team, composes cleanly with tower middleware, has first-class Maud integration via `IntoResponse`. Simpler handler model than Actix-web (plain async functions with extractors). Axum 0.8 aligns with axum-core 0.5 which is what Maud 0.27 targets. | HIGH |
| Tokio | 1.x | Async runtime | Required by Axum and reqwest. Use `features = ["full"]` for dev simplicity; narrow to `["rt-multi-thread", "macros", "net"]` later if binary size matters. | HIGH |
| Maud | 0.27.x | HTML templating | Compile-time HTML templates as Rust code. Type-safe, zero-cost abstractions, no template files to manage. Has built-in Axum integration (`features = ["axum"]`) so `Markup` implements `IntoResponse` directly. Chosen per project requirements. | HIGH |
| HTMX | 2.0.x | Frontend interactivity | Serve as a static JS file from the binary. HTMX 2.0 is stable (2.0.8 latest). Provides dynamic interactions (partial page updates, form submissions) without writing JavaScript. Served from embedded static assets, not a CDN, since this is a local app. | HIGH |

### HTMX Integration

| Technology | Version | Purpose | Why | Confidence |
|------------|---------|---------|-----|------------|
| axum-htmx | 0.8.x | HTMX header extractors/responders | Provides typed extractors for HTMX request headers (`HxRequest`, `HxTarget`, `HxBoosted`) and response headers (`HxRedirect`, `HxTrigger`). The `auto-vary` feature automatically adds correct `Vary` headers for caching. Small, focused crate. | HIGH |

### Database

| Technology | Version | Purpose | Why | Confidence |
|------------|---------|---------|-----|------------|
| rusqlite | 0.38.x | SQLite access | Best choice for a single-user local app. Synchronous API is fine here -- no concurrent writers, no connection pooling needed. Simpler than SQLx for SQLite-only use. Wrapping in `tokio::task::spawn_blocking` handles the async boundary cleanly. Bundled SQLite via `features = ["bundled"]` means zero system dependencies. | HIGH |

### Web Scraping

| Technology | Version | Purpose | Why | Confidence |
|------------|---------|---------|-----|------------|
| reqwest | 0.13.x | HTTP client | De facto Rust HTTP client. Async, handles cookies/redirects/TLS. Use `features = ["cookies"]` for session handling if needed. Shares Tokio runtime with Axum. | HIGH |
| scraper | 0.25.x | HTML parsing & CSS selectors | Standard crate for parsing HTML and querying with CSS selectors. Botanical Interests pages are server-rendered HTML, so no JavaScript rendering needed -- `scraper` is sufficient. | HIGH |

### Serialization & Data

| Technology | Version | Purpose | Why | Confidence |
|------------|---------|---------|-----|------------|
| serde | 1.x | Serialization framework | Ubiquitous. Used for config, JSON responses if needed, structuring scraped data. `features = ["derive"]`. | HIGH |
| serde_json | 1.x | JSON handling | For any JSON serialization needs (API responses, config files). | HIGH |
| chrono | 0.4.x | Date/time handling | Planting schedules require date arithmetic (frost dates, weeks-before-transplant calculations). Chrono is mature, well-integrated with rusqlite and serde. Jiff (by BurntSushi) is promising but pre-1.0 -- not worth the risk for a new project that needs reliable date math now. | MEDIUM |

### Observability

| Technology | Version | Purpose | Why | Confidence |
|------------|---------|---------|-----|------------|
| tracing | 0.1.x | Structured logging | Standard in the Tokio ecosystem. Integrates naturally with Axum via tower layers. Better than `log` crate for async code. Use `tracing-subscriber` with `fmt` feature for console output. | HIGH |
| tracing-subscriber | 0.3.x | Log output formatting | Provides the `fmt` subscriber for human-readable console logs. | HIGH |

### Static Asset Serving

| Technology | Version | Purpose | Why | Confidence |
|------------|---------|---------|-----|------------|
| tower-http | 0.6.x | HTTP middleware | Provides `ServeDir` for serving static files (CSS, HTMX JS) during development, plus CORS, compression, and request tracing middleware. Part of the tower ecosystem that Axum is built on. | HIGH |
| rust-embed | 8.x | Embed assets in binary | Embeds static files (CSS, HTMX JS, images) into the compiled binary for single-binary deployment. In debug mode serves from filesystem for hot-reload of styles. Use this over `include_dir` -- better ergonomics and axum integration examples. | MEDIUM |

### Development Tools

| Technology | Version | Purpose | Why | Confidence |
|------------|---------|---------|-----|------------|
| bacon | latest | File watcher / dev reload | Replaces cargo-watch (whose own maintainer says "use bacon"). Watches source files, runs `cargo check`/`cargo run` on changes. Configure via `bacon.toml`. | HIGH |
| cargo-nextest | latest | Test runner | Faster test execution, better output formatting than `cargo test`. Optional but recommended. | MEDIUM |

## Alternatives Considered

| Category | Recommended | Alternative | Why Not |
|----------|-------------|-------------|---------|
| Web framework | Axum | Actix-web | Actix has marginally higher raw throughput but Axum has better DX, simpler handler model, and native tower/tokio integration. Actix's actor model is unnecessary complexity for this app. |
| Web framework | Axum | Rocket | Rocket has fallen behind in ecosystem momentum. Axum has more active development and community. |
| Database | rusqlite | SQLx (sqlite) | SQLx adds async overhead unnecessary for a single-user local app. Its compile-time query checking requires a live database during builds, adding friction. Rusqlite is simpler and more direct for SQLite-only use. |
| Database | rusqlite | Diesel | Full ORM is overkill. This app has ~5 tables. Hand-written SQL with rusqlite is clearer and has less boilerplate. |
| Database | rusqlite | SeaORM | Same reasoning as Diesel -- ORM overhead not justified for this scope. |
| Templating | Maud | Askama/Tera | Maud is chosen per project requirements. It offers compile-time safety and inline Rust code. Askama (Jinja-like) is the main alternative but requires separate template files. |
| Date/time | chrono | jiff | Jiff has better timezone/DST handling design but is pre-1.0 (targeting Spring/Summer 2026). This app doesn't need complex timezone math -- Halifax, MA is one fixed location. Chrono is battle-tested. |
| Date/time | chrono | time | Chrono is the ecosystem standard with broader library integration (rusqlite, serde). The `time` crate is viable but less commonly used. |
| Scraping | scraper | headless browser (thirtyfour) | Botanical Interests pages are server-rendered HTML. No JavaScript execution needed. A headless browser would add massive complexity and binary size for zero benefit. |
| Static assets | rust-embed | memory-serve | memory-serve is newer and less proven. rust-embed is well-established with 35M+ downloads. |
| Dev watcher | bacon | cargo-watch | cargo-watch maintainer recommends bacon. Bacon has better UX, lower resource usage, and is actively developed. |

## Project Structure

```
seeds-rs/
  Cargo.toml
  bacon.toml
  src/
    main.rs              # Axum server setup, routes
    routes/
      mod.rs
      seeds.rs           # Seed inventory endpoints
      schedule.rs         # Planting schedule endpoints
      scrape.rs           # Scraping trigger endpoints
    db/
      mod.rs
      migrations.rs       # Schema setup (run on startup)
      seeds.rs            # Seed CRUD operations
      schedule.rs         # Schedule queries
    scraper/
      mod.rs
      botanical.rs        # Botanical Interests page parser
    templates/
      mod.rs              # Maud template functions (components/layouts)
      layout.rs           # Base HTML layout
      seeds.rs            # Seed inventory views
      schedule.rs         # Schedule views
      components.rs       # Reusable UI fragments
    models.rs             # Shared data types
    error.rs              # Error handling
  static/
    htmx.min.js           # HTMX 2.0.x vendored
    style.css             # App styles
  seeds.db                # SQLite database (gitignored)
```

## Cargo.toml Dependencies

```toml
[package]
name = "seeds-rs"
version = "0.1.0"
edition = "2024"

[dependencies]
# Web framework
axum = "0.8"
axum-htmx = { version = "0.8", features = ["auto-vary"] }
tokio = { version = "1", features = ["full"] }
tower-http = { version = "0.6", features = ["fs", "trace"] }

# Templating
maud = { version = "0.27", features = ["axum"] }

# Database
rusqlite = { version = "0.38", features = ["bundled"] }

# Scraping
reqwest = { version = "0.13", features = ["cookies"] }
scraper = "0.25"

# Serialization
serde = { version = "1", features = ["derive"] }
serde_json = "1"

# Date/time
chrono = { version = "0.4", features = ["serde"] }

# Logging
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["fmt"] }

# Static assets (for production binary embedding)
rust-embed = "8"
```

## Key Integration Notes

### Maud + Axum
Maud's `axum` feature makes `Markup` implement `IntoResponse`, so handlers return `Markup` directly:
```rust
async fn seed_list() -> Markup {
    html! {
        div #seed-list {
            h2 { "My Seeds" }
            // ...
        }
    }
}
```

### Maud + HTMX
HTMX attributes use hyphens which Maud handles naturally. Return HTML fragments for HTMX partial updates:
```rust
async fn add_seed(HxRequest(is_htmx): HxRequest) -> Markup {
    if is_htmx {
        // Return just the updated fragment
        html! { tr { td { "New seed row" } } }
    } else {
        // Return full page for non-HTMX requests
        full_page_layout()
    }
}
```

### rusqlite + Tokio
Since rusqlite is synchronous, wrap database calls in `spawn_blocking`:
```rust
let seeds = tokio::task::spawn_blocking(move || {
    let conn = Connection::open("seeds.db")?;
    // ... query
    Ok::<_, rusqlite::Error>(results)
}).await??;
```

A cleaner pattern is to create a `Db` wrapper struct that holds the connection and exposes async methods internally using `spawn_blocking`.

### HTMX Serving
Vendor HTMX into `static/htmx.min.js` rather than using a CDN. This is a local app -- no external dependencies at runtime. Use `rust-embed` to bake it into the binary for single-file deployment.

## Sources

- [Axum docs (0.8.8)](https://docs.rs/axum/latest/axum/) -- verified version
- [Maud docs (0.27.0)](https://docs.rs/maud/latest/maud/) -- verified version, confirmed axum-core 0.5 integration
- [rusqlite docs (0.38.0)](https://docs.rs/rusqlite/latest/rusqlite/) -- verified version
- [reqwest docs (0.13.2)](https://docs.rs/reqwest/latest/reqwest/) -- verified version
- [scraper docs (0.25.0)](https://docs.rs/scraper/latest/scraper/) -- verified version
- [axum-htmx docs (0.8.1)](https://docs.rs/axum-htmx/latest/axum_htmx/) -- verified version and features
- [tower-http docs (0.6.8)](https://docs.rs/tower-http/latest/tower_http/) -- verified version
- [tokio docs (1.50.0)](https://docs.rs/tokio/latest/tokio/) -- verified version
- [HTMX official site](https://htmx.org/) -- HTMX 2.0.x stable
- [Rust Web Frameworks in 2026 comparison](https://aarambhdevhub.medium.com/rust-web-frameworks-in-2026-axum-vs-actix-web-vs-rocket-vs-warp-vs-salvo-which-one-should-you-2db3792c79a2)
- [Rust ORMs in 2026 comparison](https://aarambhdevhub.medium.com/rust-orms-in-2026-diesel-vs-sqlx-vs-seaorm-vs-rusqlite-which-one-should-you-actually-use-706d0fe912f3)
- [MASH stack article (Maud, Axum, SQLx, HTMX)](https://emschwartz.me/building-a-fast-website-with-the-mash-stack-in-rust/)
- [HARM stack article (HTMX, Axum, Rust, Maud)](https://nguyenhuythanh.com/posts/the-harm-stack-considered-unharmful/)
- [Jiff vs Chrono comparison](https://docs.rs/jiff/latest/jiff/_documentation/comparison/index.html)
- [Bacon overview](https://dystroy.org/bacon/)
- [Rust web scraping guide 2025](https://evomi.com/blog/rust-web-scraping-2025-steps-tools-proxies)
- [serde docs (1.0.228)](https://docs.rs/crate/serde/latest)
