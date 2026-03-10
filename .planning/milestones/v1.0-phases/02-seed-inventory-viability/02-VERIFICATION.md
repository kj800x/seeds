---
phase: 02-seed-inventory-viability
verified: 2026-03-08T23:59:00Z
status: passed
score: 10/10 must-haves verified
re_verification: false
---

# Phase 2: Seed Inventory + Viability Verification Report

**Phase Goal:** Users can manage their seed collection with purchase year tracking and see how viable each seed packet remains
**Verified:** 2026-03-08T23:59:00Z
**Status:** passed
**Re-verification:** No -- initial verification

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | Seed struct has purchase_year and notes fields | VERIFIED | `models.rs` lines 29-30: `pub purchase_year: Option<i64>`, `pub notes: Option<String>` (deprecated but present; new data goes to seed_purchases) |
| 2 | Viability percentage is computed from species and seed age | VERIFIED | `viability/mod.rs` lines 25-61: `estimate_viability()` computes linear decline from `lookup_max_years()` and `chrono::Local::now().year()` |
| 3 | Species lookup table covers common vegetables, herbs, and flowers | VERIFIED | `viability/lookup.rs` has 80+ entries across vegetables (tomato, pepper, onion...), herbs (basil, dill...), flowers (zinnia, marigold...). Test `table_has_enough_entries` asserts >= 70. |
| 4 | update_seed and delete_seed queries exist and work | VERIFIED | `queries.rs` line 195: `delete_seed`. Purchase-level CRUD replaces seed-level update: `update_purchase` at line 141, `delete_purchase` at line 159. Architectural deviation documented. |
| 5 | User can see purchase year and viability percentage in the seed list | VERIFIED | `templates/home.rs` lines 55-73: renders newest purchase year, lot count, and viability percentage via `estimate_viability()` |
| 6 | User can see viability info on the seed detail page | VERIFIED | `templates/seed_detail.rs` lines 9-85: `seed_purchases_section()` renders per-lot viability with percentage, age, and max years |
| 7 | User can specify purchase year when adding a new seed | VERIFIED | `templates/home.rs` lines 20-23: purchase_year number input. `routes/seeds.rs` lines 57-59: creates purchase record via `insert_purchase` |
| 8 | User can edit purchase year and notes on the seed detail page | VERIFIED | `routes/seeds.rs` lines 127-141: `update_purchase_handler` with HTMX PUT. Edit form at lines 159-199. |
| 9 | User can delete a seed from the detail page | VERIFIED | `templates/seed_detail.rs` lines 221-226: delete button with `hx-delete` and `hx-confirm`. `routes/seeds.rs` lines 215-231: `delete_seed_handler` with image cleanup and HX-Redirect. |
| 10 | Seeds with no purchase year show a prompt to set it | VERIFIED | `templates/home.rs` lines 71-73: "Add purchase to see viability". `templates/seed_detail.rs` line 16: "No purchases recorded. Add one below to track viability." |

**Score:** 10/10 truths verified

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `seeds-rs/migrations/002_inventory.sql` | purchase_year and notes columns | VERIFIED | 2 ALTER TABLE statements adding INTEGER and TEXT columns |
| `seeds-rs/migrations/003_seed_purchases.sql` | seed_purchases table with data migration | VERIFIED | CREATE TABLE with FK to seeds, INSERT migration from deprecated columns |
| `seeds-rs/src/viability/mod.rs` | ViabilityEstimate struct and estimate_viability | VERIFIED | 167 lines with struct, public function, test helper, and 8 tests |
| `seeds-rs/src/viability/lookup.rs` | Species-to-max-years lookup table | VERIFIED | 182 lines, 80+ entries in LazyLock HashMap, lookup_max_years with SubCat normalization, 7 tests |
| `seeds-rs/src/db/queries.rs` | CRUD queries for seeds and purchases | VERIFIED | 233 lines: delete_seed, plus full purchase CRUD (insert/update/delete/list/get/count/newest) |
| `seeds-rs/src/db/models.rs` | Seed with purchase_year/notes, SeedPurchase struct | VERIFIED | Both structs present with sqlx::FromRow derive |
| `seeds-rs/src/routes/seeds.rs` | Purchase CRUD handlers, delete seed handler | VERIFIED | 232 lines: add/update/delete purchase handlers, edit form, delete seed with image cleanup |
| `seeds-rs/src/templates/home.rs` | Seed list with purchase year and viability | VERIFIED | 84 lines: renders newest year, lot count, viability percentage, prompt for missing purchases |
| `seeds-rs/src/templates/seed_detail.rs` | Detail page with viability display, purchase table, delete button | VERIFIED | 232 lines: purchase history table with per-lot viability, inline edit via HTMX, delete with confirmation |

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `templates/home.rs` | `viability/mod.rs` | `estimate_viability` call | WIRED | Line 5: `use crate::viability::estimate_viability`, line 64: called for each seed |
| `templates/seed_detail.rs` | `viability/mod.rs` | `estimate_viability` call | WIRED | Line 4: `use crate::viability::estimate_viability`, line 29: called per purchase lot |
| `routes/seeds.rs` | `db/queries.rs` | purchase CRUD calls | WIRED | Calls `insert_purchase`, `update_purchase`, `delete_purchase`, `delete_seed`, `list_purchases_for_seed` |
| `templates/seed_detail.rs` | `/seeds/{id}` | hx-delete for seed delete | WIRED | Line 223: `hx-delete` with confirmation dialog |
| `templates/seed_detail.rs` | `/seeds/{id}/purchases/{id}` | hx-put/hx-delete for purchase edit/delete | WIRED | Lines 54-61: edit and delete buttons per purchase row |
| `viability/mod.rs` | `viability/lookup.rs` | `lookup_max_years` call | WIRED | Line 4: `use lookup::lookup_max_years`, line 39: called in estimate_viability |
| `main.rs` | `routes/seeds.rs` | Route registration | WIRED | Lines 44-51: all seed and purchase routes registered |
| `main.rs` | viability module | `mod viability` | WIRED | Line 6: `mod viability;` |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|------------|-------------|--------|----------|
| INVT-01 | 02-02 | View all seeds with key details (name, variety, year, viability) | SATISFIED | `templates/home.rs` renders title, category, subcategory, days to maturity, purchase year, viability percentage |
| INVT-02 | 02-02 | View detailed seed info page with all scraped data and images | SATISFIED | `templates/seed_detail.rs` renders full detail page with images, growing info, planting, harvest, about sections |
| INVT-03 | 02-01, 02-02 | Specify purchase year when adding a seed | SATISFIED | Add form has purchase_year input; `add_seed` handler creates purchase record |
| INVT-04 | 02-01, 02-02 | Edit seed inventory entries (year, notes) | SATISFIED | Purchase-level edit via HTMX inline form with `update_purchase_handler` |
| INVT-05 | 02-01, 02-02 | Delete seeds from inventory | SATISFIED | `delete_seed_handler` with image cleanup, `delete_purchase_handler` for individual lots |
| VIAB-01 | 02-01, 02-02 | Display estimated germination viability based on species and age | SATISFIED | `estimate_viability()` returns percentage; displayed in both list and detail views |
| VIAB-02 | 02-01 | Viability uses species-based lookup table | SATISFIED | `lookup.rs` has 80+ species with subcategory-first lookup, category fallback, 2-year default |

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| `viability/mod.rs` | 64 | Unused function warning (`estimate_viability_with_year`) | Info | Test helper only used in `#[cfg(test)]` but defined outside test module. Compiler warns but does not block. |

No blockers, no stubs, no placeholder implementations found.

### Human Verification Required

### 1. End-to-End Add Seed with Purchase Year

**Test:** Paste a Botanical Interests URL with a purchase year, verify the seed appears in the list with viability percentage
**Expected:** Seed is scraped, purchase record created, viability percentage shown in both list and detail views
**Why human:** Requires running app with live scraping against external site

### 2. HTMX Inline Edit Flow

**Test:** On a seed detail page, click Edit on a purchase row, change year, save. Then click Edit again and Cancel.
**Expected:** Edit swaps to inline form, save updates and swaps back, cancel reverts without saving
**Why human:** HTMX swap behavior requires browser interaction

### 3. Delete Seed Flow

**Test:** Delete a seed from its detail page
**Expected:** Confirmation dialog appears, seed is removed, user is redirected to home page
**Why human:** Browser confirmation dialog and redirect behavior

### Architectural Note

The implementation deviated from the original Plan 02 design (single purchase_year column on seeds) to a multi-lot `seed_purchases` table, per user feedback during execution. This is a better data model that still satisfies all requirements. The Plan 02 must-haves referenced `update_seed` / `edit_seed_form` / `seed_info_fragment` handlers, but the actual implementation uses purchase-level CRUD equivalents (`update_purchase_handler`, `edit_purchase_form`, `purchases_fragment`). The observable truths remain fully met.

### Gaps Summary

No gaps found. All 10 observable truths verified, all 9 artifacts confirmed substantive and wired, all 8 key links verified, all 7 requirements satisfied. Code compiles cleanly (only minor warnings), 15 viability tests pass.

---

_Verified: 2026-03-08T23:59:00Z_
_Verifier: Claude (gsd-verifier)_
