# Phase 2: Seed Inventory + Viability - Research

**Researched:** 2026-03-08
**Domain:** Inventory CRUD with HTMX, seed viability estimation
**Confidence:** HIGH

## Summary

Phase 2 adds inventory management (purchase year tracking, edit, delete) and seed viability estimation to the existing Rust/Axum/Maud/HTMX application built in Phase 1. The codebase already has a working seed list, detail view, and scraper. This phase needs: (1) a database migration to add `purchase_year` and `notes` columns to the `seeds` table, (2) new query functions for update/delete, (3) HTMX-powered edit and delete UI on the detail page, (4) a purchase year input on the add-seed form, (5) a viability lookup table mapping species (via `subcategory` or `category`) to max viable years, and (6) a simple linear viability calculation displayed on both the list and detail views.

The viability system is the only novel domain here. The approach is a static lookup table in Rust (HashMap or match) that maps seed species/subcategory to maximum viable years, then computes a percentage based on `current_year - purchase_year`. The Botanical Interests `tags_raw` field already contains "SubCat - Tomato" style data that maps well to viability species. For species not in the table, a conservative default of 2 years is appropriate.

**Primary recommendation:** Use a SQLite migration to add `purchase_year INTEGER` and `notes TEXT` to the seeds table, build viability as a pure Rust function (no database needed), and use HTMX `hx-put` / `hx-delete` for edit/delete operations returning HTML fragments.

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| INVT-01 | User can view all seeds in inventory with key details (name, variety, year, viability indicator) | Extend existing `home_page` template to show purchase_year and computed viability percentage |
| INVT-02 | User can view detailed seed info page showing all scraped packet data and images | Already partially done in Phase 1; needs purchase_year, notes, and viability display added |
| INVT-03 | User can specify purchase year when adding a seed | Add `purchase_year` field to the add-seed form and propagate through scraper flow |
| INVT-04 | User can edit seed inventory entries (year, notes) | New PUT route with HTMX inline editing on detail page |
| INVT-05 | User can delete seeds from inventory | New DELETE route with HTMX confirmation pattern; CASCADE already handles images |
| VIAB-01 | App displays estimated germination viability percentage based on species and seed age | Pure Rust viability calculation function using lookup table + linear decline |
| VIAB-02 | Viability estimation uses a species-based lookup table of viability curves | Static HashMap mapping subcategory/category to max years with viability percentage computation |
</phase_requirements>

## Standard Stack

### Core (already in place from Phase 1)
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| axum | 0.8 | HTTP routing | Already in Cargo.toml |
| maud | 0.27 | HTML templating | Already in Cargo.toml, features = ["axum"] |
| sqlx | 0.8 | Database (SQLite) | Already in Cargo.toml, with migrations |
| htmx | 2.0.4 | Client-side interactivity | Served from /static/, already working |
| tokio | 1 | Async runtime | Already in Cargo.toml |

### Supporting (no new dependencies needed)
No new crates are required for Phase 2. All functionality (CRUD operations, viability calculation) can be built with the existing stack.

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Static Rust HashMap for viability | SQLite table for viability data | Overkill -- data is static, ~80 entries, changes rarely; Rust code is simpler and faster |
| Linear viability decline | Exponential/sigmoid model | Linear is simpler and "good enough" for a gardening app; scientific precision not needed |

## Architecture Patterns

### Database Migration (002_inventory.sql)

Add purchase_year and notes to the existing seeds table:

```sql
-- migrations/002_inventory.sql
ALTER TABLE seeds ADD COLUMN purchase_year INTEGER;
ALTER TABLE seeds ADD COLUMN notes TEXT;
```

SQLite `ALTER TABLE ADD COLUMN` is the standard approach. Each ALTER must be a separate statement. The purchase_year is nullable to preserve existing seeds (they can be edited to set it later).

### Pattern 1: HTMX Edit-in-Place

**What:** Use `hx-get` to swap a display section with an edit form, `hx-put` to save, targeting a specific element.
**When to use:** Editing seed fields (purchase_year, notes) on the detail page.
**Example:**
```html
<!-- Display mode -->
<div id="seed-info" hx-target="this" hx-swap="outerHTML">
  <span>Purchase Year: 2024</span>
  <button hx-get="/seeds/1/edit" class="btn-edit">Edit</button>
</div>

<!-- After clicking Edit, server returns: -->
<form id="seed-info" hx-put="/seeds/1" hx-target="this" hx-swap="outerHTML">
  <input type="number" name="purchase_year" value="2024" min="2000" max="2026">
  <textarea name="notes">...</textarea>
  <button type="submit">Save</button>
  <button hx-get="/seeds/1/info" type="button">Cancel</button>
</form>
```

In Maud, the server renders both the display and edit form variants as separate handler functions. The PUT handler saves to DB and returns the display mode HTML fragment.

### Pattern 2: HTMX Delete with Confirmation

**What:** Use `hx-delete` with `hx-confirm` attribute for simple browser-native confirmation.
**When to use:** Deleting a seed from inventory.
**Example:**
```html
<button hx-delete="/seeds/1"
        hx-confirm="Delete this seed from your inventory?"
        hx-target="body"
        hx-push-url="/">
  Delete Seed
</button>
```

The delete handler removes the seed (images cascade via FK), then returns an `HX-Redirect: /` header to send the user back to the list.

### Pattern 3: Viability as Pure Computation

**What:** Viability is computed at render time, not stored in the database.
**When to use:** Any time a seed is displayed (list view, detail view).
**Example:**
```rust
pub struct ViabilityEstimate {
    pub percentage: u8,      // 0-100
    pub max_years: u8,       // species max viable years
    pub age_years: u8,       // current_year - purchase_year
    pub species_key: String, // what we matched on
}

pub fn estimate_viability(subcategory: Option<&str>, category: Option<&str>, purchase_year: Option<i32>) -> Option<ViabilityEstimate> {
    let purchase_year = purchase_year?;
    let current_year = chrono::Local::now().year() as i32; // or just hardcode from system
    let age = (current_year - purchase_year).max(0) as u8;

    let max_years = lookup_max_years(subcategory, category);

    if age == 0 {
        return Some(ViabilityEstimate { percentage: 100, max_years, age_years: 0, species_key: ... });
    }
    if age >= max_years {
        return Some(ViabilityEstimate { percentage: 0, max_years, age_years: age, species_key: ... });
    }

    // Linear decline: 100% at year 0, 0% at max_years
    let pct = ((max_years as f32 - age as f32) / max_years as f32 * 100.0) as u8;
    Some(ViabilityEstimate { percentage: pct, max_years, age_years: age, species_key: ... })
}
```

**Note on year calculation:** `chrono` is not in Cargo.toml. Rather than adding a dependency, use `std::time::SystemTime` or add `chrono` minimally. Adding chrono is fine since it will be needed for Phase 3 (planting schedules) anyway.

### Recommended New File Structure
```
seeds-rs/
  migrations/
    001_initial.sql          # existing
    002_inventory.sql        # NEW: purchase_year + notes columns
  src/
    viability/
      mod.rs                 # NEW: viability module
      lookup.rs              # NEW: species -> max_years lookup table
    routes/
      seeds.rs               # MODIFY: add edit/delete/edit-form handlers
    templates/
      home.rs                # MODIFY: show purchase_year + viability in list
      seed_detail.rs         # MODIFY: show viability, add edit/delete UI
    db/
      queries.rs             # MODIFY: add update_seed, delete_seed
      models.rs              # MODIFY: add purchase_year, notes to Seed struct
```

### Anti-Patterns to Avoid
- **Storing computed viability in the database:** It depends on `current_year` which changes, so it would go stale. Always compute at render time.
- **Over-engineering the viability model:** A simple linear decline from 100% to 0% over max_years is perfectly adequate. No need for sigmoid curves or per-species calibration data.
- **Using JavaScript for edit forms:** HTMX handles the swap pattern entirely server-side; no custom JS needed.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Current year | Manual epoch math | `chrono::Local::now().year()` | Timezone handling, correctness |
| Delete cascades | Manual image cleanup | SQLite `ON DELETE CASCADE` | Already set up in migration 001 |
| Form validation | Custom parsing | Axum's `Form` extractor with Deserialize | Already the pattern in Phase 1 |
| Confirmation dialogs | Custom modal JS | `hx-confirm` attribute | Native browser dialog, zero JS |

## Common Pitfalls

### Pitfall 1: SQLite ALTER TABLE limitations
**What goes wrong:** Trying to add multiple columns in a single ALTER TABLE statement.
**Why it happens:** SQLite only supports adding one column per ALTER TABLE statement.
**How to avoid:** Use separate `ALTER TABLE ... ADD COLUMN` statements in the migration file.
**Warning signs:** Migration fails with syntax error.

### Pitfall 2: Forgetting to update the Seed struct and queries
**What goes wrong:** Adding DB columns but not updating the `Seed` struct, causing sqlx compile-time or runtime errors.
**Why it happens:** sqlx `FromRow` derive maps columns to struct fields by name.
**How to avoid:** Update `Seed` struct, `NewSeed` struct, `insert_seed` query, and all `SELECT *` queries simultaneously.
**Warning signs:** Compilation errors from sqlx macros, or runtime "missing column" errors.

### Pitfall 3: Viability mapping misses
**What goes wrong:** Seeds display no viability because their subcategory doesn't match the lookup table.
**Why it happens:** Botanical Interests subcategory tags (e.g., "SubCat - Tomato") may not exactly match lookup table keys.
**How to avoid:** Normalize lookup keys to lowercase, use contains/fuzzy matching, and provide a fallback default (2 years). Log misses so they can be added to the table.
**Warning signs:** Seeds showing "Unknown viability" or no viability indicator.

### Pitfall 4: Purchase year not captured for existing seeds
**What goes wrong:** Seeds added in Phase 1 have NULL purchase_year, so viability cannot be calculated.
**Why it happens:** Phase 1 add-seed form had no purchase_year field.
**How to avoid:** Make viability display gracefully handle NULL purchase_year ("Set purchase year to see viability"). Allow editing purchase year on existing seeds.
**Warning signs:** All existing seeds show no viability.

### Pitfall 5: HTMX swap targeting wrong element
**What goes wrong:** Edit form replaces the wrong part of the page, or the response doesn't swap correctly.
**Why it happens:** Mismatched `hx-target` and element IDs, or wrong `hx-swap` strategy.
**How to avoid:** Use `id` attributes on target elements, use `outerHTML` swap strategy, and test the round-trip (display -> edit form -> save -> display).
**Warning signs:** Page layout breaks after clicking Edit or Save.

## Code Examples

### Viability Lookup Table (core data)

Based on multiple seed viability sources (High Mowing Seeds, Finch + Folly), the lookup table for Botanical Interests subcategories:

```rust
use std::collections::HashMap;
use std::sync::LazyLock;

/// Maximum viable years by species/subcategory name (lowercase).
/// Sources: High Mowing Seeds viability chart, Finch+Folly seed viability guide.
static VIABILITY_TABLE: LazyLock<HashMap<&str, u8>> = LazyLock::new(|| {
    HashMap::from([
        // Vegetables
        ("artichoke", 5), ("arugula", 3), ("bean", 3), ("beans", 3),
        ("beet", 4), ("broccoli", 3), ("brussels sprouts", 4),
        ("cabbage", 4), ("carrot", 3), ("cauliflower", 4),
        ("celery", 5), ("celeriac", 5), ("chard", 4), ("swiss chard", 4),
        ("collards", 5), ("corn", 2), ("sweet corn", 2),
        ("cucumber", 5), ("eggplant", 4), ("endive", 5),
        ("fennel", 4), ("kale", 4), ("kohlrabi", 4),
        ("leek", 1), ("lettuce", 5), ("melon", 5),
        ("mustard", 4), ("okra", 2), ("onion", 1),
        ("parsnip", 1), ("pea", 3), ("peas", 3),
        ("pepper", 2), ("peppers", 2), ("hot pepper", 2),
        ("sweet pepper", 2), ("pumpkin", 4),
        ("radish", 5), ("rutabaga", 4), ("spinach", 2),
        ("squash", 4), ("summer squash", 4), ("winter squash", 4),
        ("tomato", 4), ("turnip", 5), ("watermelon", 4),
        // Herbs
        ("basil", 5), ("cilantro", 5), ("dill", 3),
        ("parsley", 1), ("oregano", 2), ("thyme", 3),
        ("sage", 3), ("chives", 1), ("lavender", 5),
        ("chamomile", 3), ("catnip", 5), ("savory", 3),
        // Flowers
        ("zinnia", 5), ("marigold", 2), ("sunflower", 4),
        ("cosmos", 3), ("nasturtium", 5), ("aster", 1),
        ("snapdragon", 3), ("petunia", 3), ("pansy", 2),
        ("poppy", 4), ("dahlia", 2), ("dianthus", 4),
        ("hollyhock", 3), ("sweet pea", 3), ("lobelia", 3),
        ("alyssum", 4), ("celosia", 4), ("columbine", 2),
        ("foxglove", 2), ("lupine", 2), ("larkspur", 1),
        ("impatiens", 2), ("geranium", 1), ("verbena", 1),
        ("nicotiana", 3), ("salvia", 1), ("delphinium", 1),
        ("calendula", 5), ("bachelor's button", 3),
    ])
});

const DEFAULT_MAX_YEARS: u8 = 2; // Conservative fallback

pub fn lookup_max_years(subcategory: Option<&str>, category: Option<&str>) -> u8 {
    // Try subcategory first (more specific), then category
    if let Some(sub) = subcategory {
        let key = sub.to_lowercase();
        if let Some(&years) = VIABILITY_TABLE.get(key.as_str()) {
            return years;
        }
    }
    if let Some(cat) = category {
        let key = cat.to_lowercase();
        if let Some(&years) = VIABILITY_TABLE.get(key.as_str()) {
            return years;
        }
    }
    DEFAULT_MAX_YEARS
}
```

### Update and Delete Queries

```rust
pub async fn update_seed(
    pool: &SqlitePool,
    id: i64,
    purchase_year: Option<i32>,
    notes: Option<&str>,
) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE seeds SET purchase_year = ?, notes = ? WHERE id = ?")
        .bind(purchase_year)
        .bind(notes)
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn delete_seed(pool: &SqlitePool, id: i64) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM seeds WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}
```

### Route Registration Pattern

```rust
// In main.rs, add to Router:
.route("/seeds/{id}", get(routes::seeds::seed_detail)
                      .put(routes::seeds::update_seed)
                      .delete(routes::seeds::delete_seed))
.route("/seeds/{id}/edit", get(routes::seeds::edit_seed_form))
```

### Purchase Year in Add-Seed Form

```rust
// Modified AddSeedInput
#[derive(Deserialize)]
pub struct AddSeedInput {
    pub url: String,
    pub purchase_year: Option<i32>,
}
```

The form in `home.rs` adds a purchase year number input alongside the URL field.

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `lazy_static!` macro | `std::sync::LazyLock` | Rust 1.80 (stable) | No external crate needed for lazy statics |
| Axum 0.7 path syntax `/:id` | Axum 0.8 path syntax `/{id}` | Axum 0.8 | Already using correct syntax in Phase 1 |

**Note:** The project uses `edition = "2024"` which requires Rust 1.85+. `LazyLock` is stable and available.

## Open Questions

1. **Image file cleanup on seed delete**
   - What we know: `ON DELETE CASCADE` removes image DB records automatically.
   - What's unclear: The actual image files in `data/images/{seed_id}/` are not cleaned up by the database cascade.
   - Recommendation: Add filesystem cleanup in the delete handler (`std::fs::remove_dir_all`). Low risk if it fails (orphaned files are harmless).

2. **Viability display when purchase_year is NULL**
   - What we know: Phase 1 seeds won't have purchase_year set.
   - What's unclear: Best UX for prompting the user to set it.
   - Recommendation: Show "Set purchase year to see viability" with a direct link/button to the edit form.

## Sources

### Primary (HIGH confidence)
- Project codebase: Full review of all source files in seeds-rs/src/
- [High Mowing Seeds Viability Chart](https://www.highmowingseeds.com/blog/seed-viability-chart/) - Vegetable viability data
- [Finch + Folly Seed Viability](https://www.finchandfolly.com/seed-viability) - Comprehensive vegetable, herb, and flower viability data

### Secondary (MEDIUM confidence)
- Multiple seed viability sources cross-referenced (Garden Betty, Common Sense Home, Gardeners Basics) - Herb and flower data verified across sources
- [Seeds 'n Such - Germination Rates](https://seedsnsuch.com/blogs/gardeners-greenroom/germination-rates-what-do-they-actually-mean) - General germination concepts

### Tertiary (LOW confidence)
- Linear decline model is a practical simplification; real germination decline is more complex (sigmoid/exponential). Adequate for a gardening helper app but not scientifically precise.

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - No new dependencies, extending existing patterns
- Architecture: HIGH - HTMX edit/delete is well-documented; SQLite migration is straightforward
- Viability data: HIGH - Cross-referenced across 3+ authoritative seed sources
- Viability model (linear): MEDIUM - Simplification of real-world behavior, but appropriate for use case
- Pitfalls: HIGH - Based on direct codebase analysis

**Research date:** 2026-03-08
**Valid until:** 2026-04-08 (stable domain, no fast-moving dependencies)
