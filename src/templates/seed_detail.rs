use chrono::Datelike;
use maud::{html, Markup, PreEscaped};

use crate::db::models::{Seed, SeasonPlanEvent, SeedImage, SeedPurchase};
use crate::schedule::{parse_planting_timing_from_fields, compute_indoor_timeline, compute_outdoor_timeline};
use crate::viability::estimate_viability;
use super::layout::layout;
use super::schedule::{seed_detail_timeline, seed_detail_dual_timeline};

/// Render the seed purchases section with viability per purchase.
/// Used by both the full page template and the HTMX fragment handlers.
pub fn seed_purchases_section(seed: &Seed, purchases: &[SeedPurchase]) -> Markup {
    html! {
        div #seed-purchases {
            section.detail-section {
                h2 { "Purchase History & Viability" }

                @if purchases.is_empty() {
                    p.viability-prompt { "No purchases recorded. Add one below to track viability." }
                } @else {
                    table.purchases-table {
                        thead {
                            tr {
                                th { "Year" }
                                th { "Viability" }
                                th { "Notes" }
                                th { "" }
                            }
                        }
                        tbody {
                            @for purchase in purchases {
                                @let viability = estimate_viability(
                                    seed.subcategory.as_deref(),
                                    seed.category.as_deref(),
                                    Some(purchase.purchase_year),
                                );
                                tr {
                                    td { (purchase.purchase_year) }
                                    td {
                                        @if let Some(ref est) = viability {
                                            span class=(format!("viability-display {}", est.color_tier())) { (est.percentage) "%" }
                                            " "
                                            span.viability-detail {
                                                "(" (est.age_years) "/" (est.max_years) " yr)"
                                            }
                                        } @else {
                                            span.viability-prompt { "N/A" }
                                        }
                                    }
                                    td {
                                        @if let Some(ref notes) = purchase.notes {
                                            (notes)
                                        }
                                    }
                                    td.purchase-actions {
                                        button.btn.btn-edit.btn-sm
                                               hx-get=(format!("/seeds/{}/purchases/{}/edit", seed.id, purchase.id))
                                               hx-target="closest tr"
                                               hx-swap="outerHTML" { "Edit" }
                                        button.btn.btn-delete.btn-sm
                                               hx-delete=(format!("/seeds/{}/purchases/{}", seed.id, purchase.id))
                                               hx-target="#seed-purchases"
                                               hx-swap="outerHTML"
                                               hx-confirm="Delete this purchase record?" { "Delete" }
                                    }
                                }
                            }
                        }
                    }
                }

                // Add purchase form
                form.add-purchase-form
                     hx-post=(format!("/seeds/{}/purchases", seed.id))
                     hx-target="#seed-purchases"
                     hx-swap="outerHTML" {
                    div.add-purchase-fields {
                        input type="number" name="purchase_year" placeholder="Year"
                              min="2000" max="2030" required class="purchase-year-input";
                        input type="text" name="notes" placeholder="Notes (optional)"
                              class="purchase-notes-input";
                        button type="submit" class="btn btn-save btn-sm" { "Add Purchase" }
                    }
                }
            }
        }
    }
}

pub const EVENT_TYPES: &[(&str, &str)] = &[
    ("comment", "Comment"),
    ("sow_indoor", "Sow Indoors"),
    ("sow_outdoor", "Sow Outdoors"),
    ("transplant", "Transplant"),
    ("water", "Water"),
    ("fertilize", "Fertilize"),
    ("harvest", "Harvest"),
    ("observation", "Observation"),
];

fn event_type_label(event_type: &str) -> &str {
    EVENT_TYPES.iter()
        .find(|(k, _)| *k == event_type)
        .map(|(_, v)| *v)
        .unwrap_or(event_type)
}

fn format_event_date(iso_date: &str) -> String {
    // Convert YYYY-MM-DD to MM/DD/YYYY for display
    if let Some((y, rest)) = iso_date.split_once('-') {
        if let Some((m, d)) = rest.split_once('-') {
            return format!("{}/{}/{}", m, d, y);
        }
    }
    iso_date.to_string()
}

/// Render the planting events section for a seed's current year.
pub fn seed_events_section(seed: &Seed, events: &[SeasonPlanEvent], year: i32) -> Markup {
    html! {
        div #seed-events {
            section.detail-section {
                h2 { "Planting Log " (year) }

                // Add event form — primary input is the message
                form.add-event-form
                     hx-post=(format!("/seeds/{}/events", seed.id))
                     hx-target="#seed-events"
                     hx-swap="outerHTML" {
                    div.event-compose {
                        input.event-message-input type="text" name="notes"
                              placeholder="What happened with this plant?"
                              autocomplete="off";
                        div.event-compose-options {
                            input.event-date-input type="date" name="event_date";
                            select.event-type-select name="event_type" {
                                @for (value, label) in EVENT_TYPES {
                                    option value=(value) { (label) }
                                }
                            }
                            button type="submit" class="btn btn-save btn-sm" { "Log" }
                        }
                    }
                }

                @if !events.is_empty() {
                    div.events-list {
                        @for event in events {
                            div.event-entry {
                                div.event-entry-header {
                                    span class=(format!("event-badge event-{}", event.event_type)) {
                                        (event_type_label(&event.event_type))
                                    }
                                    span.event-date { (format_event_date(&event.event_date)) }
                                    span.event-entry-actions {
                                        button.btn-icon
                                               hx-get=(format!("/seeds/{}/events/{}/edit", seed.id, event.id))
                                               hx-target=(format!("#event-{}", event.id))
                                               hx-swap="outerHTML" { "Edit" }
                                        button.btn-icon.btn-icon-delete
                                               hx-delete=(format!("/seeds/{}/events/{}", seed.id, event.id))
                                               hx-target="#seed-events"
                                               hx-swap="outerHTML"
                                               hx-confirm="Delete this entry?" { "\u{00d7}" }
                                    }
                                }
                                @if let Some(ref notes) = event.notes {
                                    @if !notes.is_empty() {
                                        p.event-entry-notes id=(format!("event-{}", event.id)) { (notes) }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Helper to render a definition list item if the value is Some.
fn detail_item(label: &str, value: &Option<String>) -> Markup {
    html! {
        @if let Some(v) = value {
            dt { (label) }
            dd { (v) }
        }
    }
}

pub fn seed_detail_page(seed: &Seed, images: &[SeedImage], purchases: &[SeedPurchase], events: &[SeasonPlanEvent], in_plan: bool, is_skipped: bool, plan_start_method: Option<&str>) -> Markup {
    let hero_image = images.iter().find(|img| img.position == 1);
    let current_year = chrono::Local::now().year();
    let timing = parse_planting_timing_from_fields(
        seed.when_to_sow_outside.as_deref(),
        seed.when_to_start_inside.as_deref(),
    );

    let indoor_timeline = compute_indoor_timeline(seed, &timing, current_year);
    let outdoor_timeline = compute_outdoor_timeline(seed, &timing, current_year);
    let has_both = indoor_timeline.is_some() && outdoor_timeline.is_some();

    let content = html! {
        div.seed-detail {
            a.back-link href="/" { "\u{2190} Back to Seeds" }

            @if let Some(img) = hero_image {
                div.hero-image {
                    img src=(format!("/images/{}/{}", seed.id, img.local_filename))
                        alt=(seed.title);
                }
            }

            h1.seed-title { (seed.title) }

            // Botanical name in italics below title
            @if let Some(ref name) = seed.botanical_name {
                p.botanical-name { em { (name) } }
            }

            div.seed-badges {
                @if seed.is_organic {
                    span.badge.badge-organic { "Organic" }
                }
                @if seed.is_heirloom {
                    span.badge.badge-heirloom { "Heirloom" }
                }
            }

            // Source link
            a.source-link href=(&seed.source_url) target="_blank" rel="noopener" {
                "View on Botanical Interests \u{2197}"
            }

            // Timeline section - show separate indoor/outdoor if both available
            @if has_both {
                (seed_detail_dual_timeline(seed, &timing, current_year, in_plan, is_skipped, plan_start_method, events))
            } @else {
                (seed_detail_timeline(seed, &timing, current_year, in_plan, is_skipped, events))
            }

            // Planting events log
            (seed_events_section(seed, events, current_year))

            // About section
            @if seed.category.is_some() || seed.subcategory.is_some() || seed.description.is_some() {
                section.detail-section {
                    h2 { "About" }
                    dl.info-list {
                        @if let Some(ref cat) = seed.category {
                            dt { "Category" }
                            dd { (cat) }
                        }
                        @if let Some(ref subcat) = seed.subcategory {
                            dt { "Subcategory" }
                            dd { (subcat) }
                        }
                    }
                    @if let Some(ref desc) = seed.description {
                        div.seed-description {
                            (PreEscaped(desc))
                        }
                    }
                }
            }

            // Sowing & Planting section
            @if seed.when_to_sow_outside.is_some() || seed.when_to_start_inside.is_some()
                || seed.sow_depth.is_some() || seed.plant_spacing.is_some()
                || seed.row_spacing.is_some() || seed.thinning.is_some()
                || seed.days_to_emerge.is_some() || seed.germination_info.is_some() {
                section.detail-section {
                    h2 { "Sowing & Planting" }
                    dl.info-list {
                        (detail_item("When to Start Inside", &seed.when_to_start_inside))
                        (detail_item("When to Sow Outside", &seed.when_to_sow_outside))
                        (detail_item("Days to Emerge", &seed.days_to_emerge))
                        (detail_item("Seed Depth", &seed.sow_depth))
                        (detail_item("Seed Spacing", &seed.plant_spacing))
                        (detail_item("Row Spacing", &seed.row_spacing))
                        (detail_item("Thinning", &seed.thinning))
                    }
                }
            }

            // Harvest section
            @if seed.harvest_instructions.is_some() {
                section.detail-section {
                    h2 { "Harvest" }
                    @if let Some(ref harvest) = seed.harvest_instructions {
                        p { (harvest) }
                    }
                }
            }

            // Plant Info section (identity/characteristics)
            @if seed.family.is_some() || seed.plant_type.is_some() || seed.hardiness.is_some()
                || seed.exposure.is_some() || seed.plant_dimensions.is_some()
                || seed.native_region.is_some() || seed.bloom_period.is_some()
                || seed.days_to_maturity.is_some() || seed.light_requirement.is_some()
                || seed.frost_tolerance.is_some() {
                section.detail-section {
                    h2 { "Plant Info" }
                    dl.info-list {
                        (detail_item("Days to Maturity", &seed.days_to_maturity))
                        (detail_item("Family", &seed.family))
                        (detail_item("Type", &seed.plant_type))
                        (detail_item("Hardiness", &seed.hardiness))
                        (detail_item("Light", &seed.light_requirement))
                        (detail_item("Exposure", &seed.exposure))
                        (detail_item("Frost Tolerance", &seed.frost_tolerance))
                        (detail_item("Plant Dimensions", &seed.plant_dimensions))
                        (detail_item("Bloom Period", &seed.bloom_period))
                        (detail_item("Native", &seed.native_region))
                        (detail_item("Attributes", &seed.attributes))
                    }
                }
            }

            // Variety Info section
            @if seed.variety_info.is_some() {
                section.detail-section {
                    h2 { "Variety Info" }
                    @if let Some(ref info) = seed.variety_info {
                        p { (info) }
                    }
                }
            }

            // Special Care section
            @if seed.special_care.is_some() {
                section.detail-section {
                    h2 { "Special Care" }
                    @if let Some(ref care) = seed.special_care {
                        p { (care) }
                    }
                }
            }

            // Delete button
            div.delete-section {
                button.btn.btn-delete
                       hx-delete=(format!("/seeds/{}", seed.id))
                       hx-confirm="Delete this seed from your inventory? This cannot be undone."
                       hx-target="body" { "Delete Seed" }
            }
        }
    };

    layout(&seed.title, content)
}
