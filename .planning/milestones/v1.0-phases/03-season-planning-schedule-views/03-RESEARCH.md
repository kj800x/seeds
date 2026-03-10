# Phase 3: Season Planning + Schedule Views - Research

**Researched:** 2026-03-08
**Domain:** Planting date calculation from scraped free-text data + schedule UI (Maud/HTMX)
**Confidence:** HIGH

## Summary

This phase adds the core value proposition of the app: selecting seeds to grow this season and generating a complete planting schedule with start-indoors and transplant-outdoors dates. The technical challenge breaks into three areas: (1) parsing planting timing data from free-form scraped text, (2) computing actual calendar dates from relative frost-date references, and (3) rendering the schedule as both a sorted action list and a visual timeline.

The existing `planting_instructions` field contains the raw scraped text from Botanical Interests, which consistently uses phrases like "4 to 6 weeks before your average last frost date" and "1 to 2 weeks after your average last frost date." The `frost_tolerance` tag from tags_raw distinguishes cool-season from warm-season crops. The `days_to_maturity` field provides harvest timing. All date math is relative to Halifax MA's last frost date (~May 10).

**Primary recommendation:** Build a `schedule` module with a planting text parser that extracts weeks-before/after-frost numbers via regex, a date calculator that resolves these to calendar dates, and a new `season_plans` DB table linking seeds to seasons. Render schedule views as Maud templates with HTMX interactions, using pure CSS for the timeline visualization.

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| PLAN-01 | User can select which seeds from inventory to grow this season | New `season_plans` table + toggle UI on seed list/detail pages |
| PLAN-02 | App generates planting schedule with start-indoors dates based on Halifax MA last frost (~May 10) | Regex parser extracts "X weeks before last frost" from `planting_instructions`, date calculator applies to May 10 |
| PLAN-03 | App generates transplant-outdoors dates based on frost dates and seed packet data | Parser extracts "X weeks after last frost" transplant timing from same text |
| PLAN-04 | Schedule handles cool-season crops (before last frost) and warm-season crops (after last frost) | `frost_tolerance` tag + `planting_instructions` text both encode this; parser must handle both patterns |
| PLAN-05 | Planting dates derived from scraped seed packet data | All timing comes from `planting_instructions` free text already in DB |
| VIEW-01 | User can view planting schedule as sorted action list with dates | New `/schedule` route rendering Maud template, sorted by date |
| VIEW-02 | User can view planting schedule as visual calendar/timeline | Pure CSS horizontal timeline showing season blocks per seed |
</phase_requirements>

## Standard Stack

### Core (already in project)
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| axum | 0.8 | HTTP routes for schedule pages | Already used for all routes |
| maud | 0.27 | Server-side HTML for schedule views | Already used for all templates |
| sqlx | 0.8 (sqlite) | Season plans table, schedule queries | Already used for all DB access |
| chrono | 0.4 | Date arithmetic for frost date calculations | Already a dependency |
| HTMX | 2.0.4 | Toggle seeds in/out of plan, dynamic schedule updates | Already served from /static/ |

### No new dependencies needed
This phase requires no additional crates. The `chrono` crate already provides `NaiveDate`, date arithmetic, and formatting. Regex-like parsing can use standard Rust string methods (`contains`, `find`, `split`) since the patterns are predictable and limited in number.

## Architecture Patterns

### New Module Structure
```
src/
├── schedule/              # NEW - all planting schedule logic
│   ├── mod.rs             # Public API: generate_schedule(), PlantingAction
│   ├── parser.rs          # Extract timing from planting_instructions text
│   └── calculator.rs      # Convert relative timing to calendar dates
├── routes/
│   ├── schedule.rs        # NEW - /schedule, /plan routes
│   └── seeds.rs           # MODIFY - add plan toggle endpoint
├── templates/
│   ├── schedule.rs        # NEW - action list + timeline views
│   └── home.rs            # MODIFY - add "plan this season" toggle
└── db/
    ├── queries.rs         # MODIFY - add season_plan CRUD + schedule queries
    └── models.rs          # MODIFY - add SeasonPlan model
```

### Pattern 1: Planting Text Parser
**What:** Regex-style extraction of relative timing from free-form planting text
**When to use:** When generating schedule from scraped data

Actual patterns found in the database:

| Crop Type | Example Text | Extract |
|-----------|-------------|---------|
| Warm-season (tomato) | "4 to 6 weeks before transplanting. Transplant...1 to 2 weeks after your average last frost date" | start_indoors: 6 weeks before last frost, transplant: 1 week after |
| Warm-season (bean) | "1 to 2 weeks after your average last frost date" (outdoor only) | direct_sow: 1 week after last frost |
| Cool-season (lettuce) | "2 to 4 weeks before your average last frost date" | direct_sow: 4 weeks before last frost |
| Herbs (anise hyssop) | "4 to 6 weeks before your average last frost date" (start inside) | start_indoors: 6 weeks before last frost |
| Long-start (rosemary) | "10 to 12 weeks before your average last frost date" | start_indoors: 10 weeks before last frost |

```rust
/// Parsed planting timing from free-text instructions.
pub struct PlantingTiming {
    /// Weeks before last frost to start indoors (None = not recommended indoors)
    pub start_indoors_weeks_before: Option<u8>,
    /// Weeks before/after last frost to transplant outdoors (negative = before)
    pub transplant_weeks_relative: Option<i8>,
    /// Weeks before/after last frost for direct sow (negative = before)
    pub direct_sow_weeks_relative: Option<i8>,
    /// Whether starting indoors is recommended
    pub indoor_start_recommended: bool,
    /// Days to emergence/germination
    pub days_to_emerge: Option<u8>,
}

/// Parse planting timing from the planting_instructions text field.
pub fn parse_planting_timing(text: &str) -> PlantingTiming {
    let lower = text.to_lowercase();
    // Key phrases to match:
    // "X to Y weeks before your average last frost date" -> use Y (conservative)
    // "X to Y weeks after your average last frost date" -> use X (earliest safe)
    // "RECOMMENDED" near "Start Inside" -> indoor_start_recommended = true
    // ...
}
```

### Pattern 2: Date Calculator
**What:** Convert relative frost-date timing into actual calendar dates
**When to use:** After parsing, to generate the schedule

```rust
use chrono::NaiveDate;

pub const HALIFAX_MA_LAST_FROST: (u32, u32) = (5, 10);  // May 10
pub const HALIFAX_MA_FIRST_FROST: (u32, u32) = (10, 15); // Oct 15

pub struct PlantingAction {
    pub seed_id: i64,
    pub seed_title: String,
    pub action_type: ActionType,  // StartIndoors, TransplantOutdoors, DirectSow
    pub date: NaiveDate,
    pub notes: String,  // e.g. "6 weeks before last frost"
}

pub enum ActionType {
    StartIndoors,
    TransplantOutdoors,
    DirectSow,
}

pub fn calculate_schedule(
    seed: &Seed,
    timing: &PlantingTiming,
    year: i32,
) -> Vec<PlantingAction> {
    let last_frost = NaiveDate::from_ymd_opt(year, HALIFAX_MA_LAST_FROST.0, HALIFAX_MA_LAST_FROST.1).unwrap();
    // Compute dates relative to last_frost using chrono::Duration::weeks()
    // ...
}
```

### Pattern 3: Season Plan Toggle (HTMX)
**What:** Checkbox/toggle on seed list to add/remove seeds from this season's plan
**When to use:** Home page seed list and seed detail page

```rust
// Route: POST /plan/toggle/{seed_id}
// Returns updated toggle button via HTMX swap
pub async fn toggle_plan(
    State(state): State<AppState>,
    Path(seed_id): Path<i64>,
) -> Result<Markup, AppError> {
    // Toggle seed in/out of current season plan
    // Return updated button reflecting new state
}
```

### Pattern 4: Timeline View (Pure CSS)
**What:** Horizontal timeline showing season span per seed
**When to use:** VIEW-02 visual calendar

Use CSS Grid with columns representing weeks. Each seed gets a row. Colored bars span from start-indoors through transplant through harvest. No JavaScript charting library needed -- this is a fixed-width grid that Maud can render with inline style widths.

```rust
// In template: each seed row has positioned bars
div.timeline-row {
    div.timeline-bar.start-indoors style=(format!("left: {}%; width: {}%", start_pct, indoor_width_pct)) {}
    div.timeline-bar.transplant style=(format!("left: {}%; width: {}%", transplant_pct, outdoor_width_pct)) {}
}
```

### Anti-Patterns to Avoid
- **Parsing all possible BI text formats:** Limit to the 5-6 patterns actually seen in the data. Log warnings for unparseable text rather than building an overly complex parser.
- **Storing computed dates in the DB:** Dates should be computed at render time (same pattern as viability). They change if the user changes the season year.
- **JavaScript charting library for timeline:** Overkill for a simple horizontal timeline. Pure CSS Grid is sufficient and keeps the no-JS-build approach intact.
- **Creating a separate "seasons" table with year management:** For v1, assume current year. A simple `season_plans` table with seed_id + year is enough.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Date arithmetic | Manual day counting | `chrono::NaiveDate` + `Duration::weeks()` | Handles month boundaries, leap years |
| Date formatting | Manual string formatting | `chrono::NaiveDate::format()` | Locale-aware, consistent |
| HTML templating | Raw string HTML | Maud macros | Already established pattern, type-safe |

**Key insight:** The app already uses chrono. NaiveDate is the right type since all dates are local (no timezone complexity for planting dates).

## Common Pitfalls

### Pitfall 1: Planting Text Parsing Fragility
**What goes wrong:** Regex breaks on unexpected BI text format variations
**Why it happens:** Free-text scraping produces inconsistent data; BI may change wording
**How to avoid:** Parse conservatively with fallback. If parsing fails, show the raw planting_instructions text and let the user see it. Never silently skip a seed from the schedule -- show it with "manual review needed" status.
**Warning signs:** Seeds appearing in plan but missing from schedule

### Pitfall 2: Confusing "Before Transplanting" with "Before Last Frost"
**What goes wrong:** Tomato text says "4 to 6 weeks before transplanting" (for indoor start), then separately "1 to 2 weeks after your average last frost date" (for transplant). If you parse "before" and assume it means before frost, dates are wrong.
**Why it happens:** The two-phase warm-season pattern (start indoors, then transplant after frost) uses "before transplanting" not "before frost" for the indoor start.
**How to avoid:** Parse the full context. For indoor start: if text says "before transplanting", find the transplant date first, then subtract. If text says "before your average last frost date", subtract from frost date directly.
**Warning signs:** Tomato indoor start dates appearing in late April instead of late March

### Pitfall 3: Cool-Season vs Warm-Season Misclassification
**What goes wrong:** A seed without `Frost Tolerant` tag gets classified as warm-season when it should be cool-season, or vice versa
**Why it happens:** Not all BI products are consistently tagged
**How to avoid:** Use BOTH the `frost_tolerance` tag AND the planting text. The text is authoritative -- if it says "before your average last frost date" for outdoor sowing, it's cool-season regardless of tag. The tag is a useful secondary signal.

### Pitfall 4: Range Interpretation (e.g., "4 to 6 weeks")
**What goes wrong:** Using the wrong end of the range produces too-early or too-late dates
**Why it happens:** "4 to 6 weeks before frost" -- do you use 4 or 6?
**How to avoid:** For "before" dates, use the larger number (start earlier = safer, more indoor growing time). For "after" dates, use the smaller number (plant sooner once safe). Document this convention clearly.

### Pitfall 5: Seeds With No Parseable Timing Data
**What goes wrong:** Seed is added to plan but parser can't extract any timing
**Why it happens:** Some seeds have minimal or unusual planting text; perennials may not follow the annual frost-date pattern
**How to avoid:** Always show these seeds in the schedule with "Could not determine dates -- see packet instructions" and link to the seed detail page where raw text is visible.

## Code Examples

### Database Migration (004_season_plans.sql)
```sql
CREATE TABLE IF NOT EXISTS season_plans (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    seed_id INTEGER NOT NULL REFERENCES seeds(id) ON DELETE CASCADE,
    year INTEGER NOT NULL,
    notes TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(seed_id, year)
);
```

### Parsing "X to Y weeks before/after" Pattern
```rust
use chrono::NaiveDate;

/// Extract the first "X to Y weeks before/after ... last frost" pattern from text.
/// Returns (weeks, is_before_frost).
fn extract_weeks_relative_to_frost(text: &str) -> Option<(u8, bool)> {
    let lower = text.to_lowercase();
    // Look for "N to M weeks before your average last frost"
    // or "N to M weeks after your average last frost"
    // Pattern: digit(s), "to", digit(s), "weeks", "before"|"after", ..., "last frost"

    let frost_idx = lower.find("last frost")?;
    let preceding = &lower[..frost_idx];

    let is_before = preceding.contains("before");
    let is_after = preceding.contains("after");
    if !is_before && !is_after {
        return None;
    }

    // Find "N to M weeks" pattern in the preceding text
    // Extract the numbers, use larger for "before", smaller for "after"
    // ... (regex or manual parsing)

    None // placeholder
}
```

### Schedule Generation Flow
```rust
pub fn generate_schedule_for_season(
    seeds: &[(Seed, PlantingTiming)],
    year: i32,
) -> Vec<PlantingAction> {
    let last_frost = NaiveDate::from_ymd_opt(year, 5, 10).unwrap();
    let mut actions = Vec::new();

    for (seed, timing) in seeds {
        if let Some(weeks) = timing.start_indoors_weeks_before {
            let date = last_frost - chrono::Duration::weeks(weeks as i64);
            actions.push(PlantingAction {
                seed_id: seed.id,
                seed_title: seed.title.clone(),
                action_type: ActionType::StartIndoors,
                date,
                notes: format!("{} weeks before last frost", weeks),
            });
        }

        if let Some(rel_weeks) = timing.transplant_weeks_relative {
            let date = if rel_weeks >= 0 {
                last_frost + chrono::Duration::weeks(rel_weeks as i64)
            } else {
                last_frost - chrono::Duration::weeks((-rel_weeks) as i64)
            };
            actions.push(PlantingAction {
                seed_id: seed.id,
                seed_title: seed.title.clone(),
                action_type: ActionType::TransplantOutdoors,
                date,
                notes: format!("{} weeks {} last frost",
                    rel_weeks.abs(),
                    if rel_weeks >= 0 { "after" } else { "before" }),
            });
        }

        if let Some(rel_weeks) = timing.direct_sow_weeks_relative {
            let date = if rel_weeks >= 0 {
                last_frost + chrono::Duration::weeks(rel_weeks as i64)
            } else {
                last_frost - chrono::Duration::weeks((-rel_weeks) as i64)
            };
            actions.push(PlantingAction {
                seed_id: seed.id,
                seed_title: seed.title.clone(),
                action_type: ActionType::DirectSow,
                date,
                notes: format!("{} weeks {} last frost",
                    rel_weeks.abs(),
                    if rel_weeks >= 0 { "after" } else { "before" }),
            });
        }
    }

    actions.sort_by_key(|a| a.date);
    actions
}
```

### Route Registration (main.rs additions)
```rust
// Add to Router::new() chain:
.route("/schedule", get(routes::schedule::schedule_page))
.route("/plan/toggle/{seed_id}", post(routes::schedule::toggle_plan))
```

### Timeline CSS Approach
```css
.timeline {
    display: grid;
    grid-template-columns: 200px repeat(26, 1fr); /* seed name + ~26 weeks (March-Sept) */
    gap: 2px 0;
}
.timeline-header { font-size: 0.75rem; text-align: center; }
.timeline-row { display: contents; }
.timeline-seed-name { padding: 4px 8px; white-space: nowrap; }
.timeline-bar {
    position: relative;
    height: 20px;
    border-radius: 3px;
}
.timeline-bar.start-indoors { background: #7c9ae0; }
.timeline-bar.transplant { background: #5cb85c; }
.timeline-bar.direct-sow { background: #f0ad4e; }
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `chrono::Duration::days()` | `chrono::TimeDelta::try_weeks()` | chrono 0.4.35+ | `Duration::weeks()` still works but `TimeDelta` is the newer type name |

**Note:** `chrono::Duration` and `chrono::TimeDelta` are the same type (TimeDelta is a type alias introduced for clarity). Both work fine with chrono 0.4.

## Open Questions

1. **Warm-season indoor start "before transplanting" parse accuracy**
   - What we know: Tomato text says "4 to 6 weeks before transplanting" separately from "transplant 1 to 2 weeks after last frost"
   - What's unclear: Whether ALL warm-season BI products use this two-phase phrasing consistently
   - Recommendation: Handle both patterns ("before transplanting" and "before last frost") in parser; test against all 4 current seeds in DB; add more BI seeds during development to validate

2. **Seeds with no timing data at all**
   - What we know: Some seeds (especially perennials like rosemary) have unconventional timing
   - What's unclear: How many BI products lack frost-relative timing
   - Recommendation: Show in schedule with "manual" status; don't block schedule generation

3. **"Days to maturity" interpretation**
   - What we know: Tomato says "75-80 days from transplanting"; bean says "58 days" (from direct sow)
   - What's unclear: Whether "from transplanting" vs "from sowing" is consistently stated
   - Recommendation: Parse the full days_to_maturity text, not just the number. If "from transplanting" is present, add to transplant date; otherwise add to sow date. This enables showing estimated harvest dates in the timeline.

## Sources

### Primary (HIGH confidence)
- **Existing codebase:** `seeds-rs/src/scraper/parser.rs` - Confirmed tag parsing for frost_tolerance
- **Existing database:** 4 scraped seeds with actual `planting_instructions` text examined
- **Existing codebase:** `seeds-rs/src/viability/mod.rs` - Pattern for computed-at-render-time approach
- **Existing codebase:** `seeds-rs/src/templates/layout.rs` - Nav already has "Schedule" placeholder
- **chrono crate:** NaiveDate, Duration/TimeDelta for date arithmetic (already a dependency)

### Secondary (MEDIUM confidence)
- **Botanical Interests website patterns** - Verified "X to Y weeks before/after last frost" phrasing across multiple product pages via web search
- **Halifax MA frost dates** - May 10 last frost, Oct 15 first frost (from PROJECT.md, standard zone 6b data)

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - No new dependencies needed, all tools already in project
- Architecture: HIGH - Follows established patterns (computed at render time, Maud templates, HTMX interactions, SQLite storage)
- Planting text parsing: MEDIUM - Validated against 4 real seeds but free-text parsing is inherently fragile; parser should be tested against more seeds during implementation
- Pitfalls: HIGH - Based on examining actual scraped data and identifying real ambiguities

**Research date:** 2026-03-08
**Valid until:** Indefinite for architecture; parsing patterns should be revalidated if Botanical Interests changes their site format
