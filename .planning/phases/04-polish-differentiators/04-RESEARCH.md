# Phase 4: Polish + Differentiators - Research

**Researched:** 2026-03-08
**Domain:** Viability UI enhancement, schedule filtering, CSS print media
**Confidence:** HIGH

## Summary

Phase 4 adds polish features to an existing Rust (Axum + Maud + HTMX) seed scheduling app. The five requirements break into two groups: viability enhancements (VIAB-03/04/05) that build on the existing `viability` module, and schedule view improvements (VIEW-03/04) that build on the existing `schedule` and `templates::schedule` modules.

All work is pure frontend/template logic with minimal backend changes. The viability color coding (VIAB-03) is a CSS class change in the existing `home.rs` template. The "use or lose" alerts (VIAB-04) and over-sowing suggestions (VIAB-05) require new helper functions in the `viability` module and template additions. The "this week" view (VIEW-03) filters existing `PlantingAction` data by date range. Print support (VIEW-04) uses CSS `@media print` rules.

**Primary recommendation:** Implement as a single plan -- all five requirements are small, independent template/CSS changes with no schema changes needed.

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| VIAB-03 | Inventory list shows color-coded viability indicators (green/yellow/orange/red) | Map percentage thresholds to CSS classes on existing `.viability` span in `home.rs` |
| VIAB-04 | App warns about seeds nearing end of useful life | Add helper fn in `viability/mod.rs` returning warning status; render alert banner in seed list/detail |
| VIAB-05 | App suggests over-sowing for reduced germination | Add `suggested_sow_multiplier()` fn based on viability percentage; display in seed detail and schedule |
| VIEW-03 | "This week" focused view showing current and upcoming actions | Filter `PlantingAction` vec by date range (today..today+14d); new route `/schedule/week` or tab on schedule page |
| VIEW-04 | Print planting schedule in print-friendly format | CSS `@media print` stylesheet hiding nav/chrome; optional dedicated print route |
</phase_requirements>

## Standard Stack

### Core (already in project)
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| axum | 0.8 | HTTP routing | Already the project's web framework |
| maud | 0.27 | HTML templating | Already used for all templates |
| chrono | 0.4 | Date math | Already used for schedule calculations |
| HTMX | 2.0.4 | Dynamic UI | Already served from /static/ |

### Supporting
No new dependencies needed. All five requirements are implementable with existing stack.

## Architecture Patterns

### Existing Project Structure (relevant files)
```
seeds-rs/src/
├── viability/
│   ├── mod.rs           # estimate_viability() -> ViabilityEstimate
│   └── lookup.rs        # species max_years table
├── schedule/
│   ├── calculator.rs    # generate_schedule() -> Vec<PlantingAction>
│   └── parser.rs        # parse_planting_timing()
├── templates/
│   ├── home.rs          # seed list with viability display
│   ├── schedule.rs      # action list + timeline views
│   └── layout.rs        # layout_with_nav()
├── routes/
│   ├── home.rs          # GET / handler
│   └── schedule.rs      # GET /schedule handler
└── static/
    └── style.css        # all styles
```

### Pattern 1: Viability Color Mapping (VIAB-03)
**What:** Map viability percentage to a color tier CSS class
**When to use:** Anywhere viability is displayed in the seed list
**Example:**
```rust
// In viability/mod.rs - add to ViabilityEstimate
pub fn color_tier(&self) -> &'static str {
    match self.percentage {
        75..=100 => "viability-green",
        50..=74  => "viability-yellow",
        25..=49  => "viability-orange",
        _        => "viability-red",
    }
}
```
```rust
// In templates/home.rs - change the existing viability span
span class=(format!("viability {}", est.color_tier())) {
    (est.percentage) "% viable"
}
```
```css
/* In style.css */
.viability-green  { color: #3d8b3d; }
.viability-yellow { color: #b8a020; }
.viability-orange { color: #c27853; }
.viability-red    { color: #c25353; font-weight: 600; }
```

### Pattern 2: Viability Warnings (VIAB-04)
**What:** Detect seeds nearing end of useful life and show alerts
**When to use:** Seeds in last year of viability or at 0%
**Example:**
```rust
// In viability/mod.rs
pub fn warning_message(&self) -> Option<String> {
    if self.percentage == 0 {
        Some(format!("These seeds have exceeded their expected viability of {} years. Consider replacing.", self.max_years))
    } else if self.age_years + 1 >= self.max_years {
        Some(format!("Last year of expected viability. Use this season or plan to replace."))
    } else {
        None
    }
}
```

### Pattern 3: Over-sowing Suggestion (VIAB-05)
**What:** Calculate a sowing multiplier to compensate for reduced germination
**When to use:** When viability is below 100%
**Example:**
```rust
// In viability/mod.rs
pub fn sow_multiplier(&self) -> Option<f32> {
    if self.percentage >= 90 {
        None // No adjustment needed
    } else if self.percentage == 0 {
        None // Don't bother sowing
    } else {
        // Sow 100/percentage times normal amount
        Some(100.0 / self.percentage as f32)
    }
}
// Display: "Sow 1.5x the normal amount to compensate for reduced germination"
```

### Pattern 4: This Week View (VIEW-03)
**What:** Filter existing schedule actions to a ~2 week window
**When to use:** Focused dashboard view for current tasks
**Approach:** Reuse `generate_schedule()` output, filter by date range, render with a dedicated template function. Can be a new route `/schedule/week` or an HTMX tab on the existing schedule page.
```rust
// In routes/schedule.rs - new handler
pub async fn this_week(State(state): State<AppState>) -> Result<Markup, AppError> {
    // ... same schedule generation as schedule_page ...
    let today = Local::now().date_naive();
    let window_end = today + Duration::days(14);
    let upcoming: Vec<_> = actions.iter()
        .filter(|a| a.date >= today && a.date <= window_end)
        .collect();
    Ok(templates::schedule::this_week_template(&upcoming))
}
```

### Pattern 5: Print Stylesheet (VIEW-04)
**What:** CSS `@media print` rules that hide navigation and optimize layout for printing
**When to use:** User triggers browser print (Ctrl+P) from schedule page
**Approach:** Pure CSS, no JavaScript or new routes needed.
```css
@media print {
    .app-header, .btn-plan-toggle, .add-seed { display: none; }
    .content { max-width: none; padding: 0; }
    .schedule-section { box-shadow: none; border: none; }
    body { font-size: 12pt; }
    a { color: inherit; text-decoration: none; }
    .timeline-today { display: none; }
}
```

### Anti-Patterns to Avoid
- **Over-engineering the color system:** Don't create a full design token system. Four CSS classes with hardcoded colors matching the existing earthy theme is correct.
- **JavaScript-based print:** Don't add a JS print library. Browser `@media print` CSS is sufficient for this use case.
- **Separate print page:** Don't create a separate route for print. CSS media queries on the existing schedule page are cleaner and always in sync.
- **Complex over-sow calculations:** Don't model germination curves. A simple `100/percentage` multiplier rounded to one decimal is adequate for a gardening helper.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Print styling | JavaScript print library / PDF generation | CSS `@media print` | Browser-native, zero dependencies, always in sync with screen version |
| Color accessibility | Custom contrast calculations | Pre-tested color values against cream background | Four fixed colors are easy to test manually |

## Common Pitfalls

### Pitfall 1: Viability Color Threshold Boundaries
**What goes wrong:** Off-by-one errors at tier boundaries (e.g., 75% could be green or yellow)
**Why it happens:** Rust match ranges are inclusive, easy to overlap or gap
**How to avoid:** Use non-overlapping ranges: 75..=100, 50..=74, 25..=49, 0..=24. Test boundary values.
**Warning signs:** Seeds at exactly 75%, 50%, 25% showing wrong color

### Pitfall 2: "This Week" Empty State
**What goes wrong:** Showing a blank page when no actions are in the current 2-week window
**Why it happens:** Most of the growing season is idle periods between actions
**How to avoid:** Show "No actions this week" with the next upcoming action date. Always show something useful.
**Warning signs:** User sees empty view most weeks

### Pitfall 3: Print Layout Breaking Timeline
**What goes wrong:** CSS grid timeline renders poorly on paper (overflow, tiny text)
**Why it happens:** Timeline uses percentage-based positioning that doesn't translate to fixed page widths
**How to avoid:** Hide the timeline in print and show only the action list. The list format prints cleanly.
**Warning signs:** Test print preview before considering done

### Pitfall 4: Over-sow for Zero Viability
**What goes wrong:** Division by zero or suggesting "sow infinity seeds"
**Why it happens:** `100 / 0` when percentage is 0
**How to avoid:** Return None for 0% viability with message "Seeds likely no longer viable"
**Warning signs:** NaN or panic in sow_multiplier calculation

### Pitfall 5: Stale "This Week" Data
**What goes wrong:** The "this week" view shows different data than the full schedule
**Why it happens:** Copying schedule generation logic instead of reusing it
**How to avoid:** Both views MUST call the same `generate_schedule()` function, then filter
**Warning signs:** Actions appearing in one view but not the other

## Code Examples

### Existing Viability Display (in home.rs, line 81-90)
Currently shows flat green text for all viability levels:
```rust
@let viability = estimate_viability(
    seed.subcategory.as_deref(),
    seed.category.as_deref(),
    newest_year,
);
@if let Some(ref est) = viability {
    span.viability { (est.percentage) "% viable" }
}
```
Change to color-coded:
```rust
@if let Some(ref est) = viability {
    span class=(format!("viability {}", est.color_tier())) {
        (est.percentage) "% viable"
    }
}
```

### Existing Schedule Route (routes/schedule.rs)
The handler generates all actions then passes to template. The "this week" view follows the same pattern but adds a date filter before template rendering.

### Nav Enhancement for "This Week" Tab
The existing nav has Seeds and Schedule. The "this week" could be:
- A sub-tab within the Schedule page (HTMX swap, no full page load)
- A separate nav item (simpler, but clutters nav)

**Recommendation:** Add as tabs within the schedule page using HTMX. Keep the nav clean with just Seeds/Schedule.

```rust
// In schedule template - add tabs at top
div.schedule-tabs {
    a.tab href="/schedule" hx-get="/schedule" hx-target=".schedule-content"
        class=@if view == "full" { "active" } { "Full Season" }
    a.tab href="/schedule/week" hx-get="/schedule/week" hx-target=".schedule-content"
        class=@if view == "week" { "active" } { "This Week" }
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| JS print libraries (jsPDF etc.) | CSS @media print | Always been standard | Zero deps, always in sync |
| Custom color pickers for status | Simple tier-based CSS classes | Standard practice | Predictable, accessible |

## Open Questions

1. **Viability threshold values (75/50/25)**
   - What we know: These are reasonable general thresholds
   - What's unclear: Whether the user wants different breakpoints
   - Recommendation: Use 75/50/25 as defaults. Easy to adjust later -- it's just a match statement.

2. **"This week" window size**
   - What we know: "Current and upcoming week" implies ~14 days
   - What's unclear: Should it include past-due actions (actions whose date has passed but may not be done)?
   - Recommendation: Show today through +14 days, plus any past actions from the current week (Mon-Sun). Include overdue actions with a visual indicator.

3. **Over-sow display location**
   - What we know: Should be visible when planning
   - What's unclear: Show on seed list, seed detail, schedule, or all?
   - Recommendation: Show on seed detail page (most context) and as a note in schedule action rows (most actionable).

## Sources

### Primary (HIGH confidence)
- Project source code: `seeds-rs/src/` -- all architecture patterns derived from existing code
- Existing CSS: `seeds-rs/static/style.css` -- design system variables and patterns
- Existing viability module: `seeds-rs/src/viability/mod.rs` -- ViabilityEstimate struct and linear model

### Secondary (MEDIUM confidence)
- CSS @media print: Standard CSS feature, well-documented across all browsers
- Maud templating patterns: Consistent with existing project usage

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - no new dependencies, extending existing code
- Architecture: HIGH - all patterns follow established project conventions
- Pitfalls: HIGH - based on direct code analysis of existing implementation

**Research date:** 2026-03-08
**Valid until:** 2026-04-08 (stable -- no external dependencies changing)
