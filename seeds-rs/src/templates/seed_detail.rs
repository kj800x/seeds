use maud::{html, Markup, PreEscaped};

use crate::db::models::{Seed, SeedImage, SeedPurchase};
use crate::viability::estimate_viability;
use super::layout::layout;

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
                        input type="number" name="purchase_year" placeholder="Year (e.g. 2025)"
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

pub fn seed_detail_page(seed: &Seed, images: &[SeedImage], purchases: &[SeedPurchase]) -> Markup {
    let hero_image = images.iter().find(|img| img.position == 1);

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

            div.seed-badges {
                @if seed.is_organic {
                    span.badge.badge-organic { "Organic" }
                }
                @if seed.is_heirloom {
                    span.badge.badge-heirloom { "Heirloom" }
                }
            }

            // Purchase history & viability section
            (seed_purchases_section(seed, purchases))

            // Viability warnings and sow multiplier
            @if !purchases.is_empty() {
                @let newest_purchase_year = purchases.iter().map(|p| p.purchase_year).max();
                @let newest_viability = estimate_viability(
                    seed.subcategory.as_deref(),
                    seed.category.as_deref(),
                    newest_purchase_year,
                );
                @if let Some(ref est) = newest_viability {
                    @if let Some(warning) = est.warning_message() {
                        div.viability-warning { (warning) }
                    }
                    @if let Some(mult) = est.sow_multiplier() {
                        div.sow-suggestion {
                            "Sow " (format!("{:.1}", mult)) "x the normal amount to compensate for reduced germination (" (est.percentage) "% viability)."
                        }
                    }
                }
            }

            // Growing Info section
            @if seed.days_to_maturity.is_some() || seed.light_requirement.is_some() || seed.frost_tolerance.is_some() {
                section.detail-section {
                    h2 { "Growing Info" }
                    dl.info-list {
                        @if let Some(ref dtm) = seed.days_to_maturity {
                            dt { "Days to Maturity" }
                            dd { (dtm) }
                        }
                        @if let Some(ref light) = seed.light_requirement {
                            dt { "Light" }
                            dd { (light) }
                        }
                        @if let Some(ref frost) = seed.frost_tolerance {
                            dt { "Frost Tolerance" }
                            dd { (frost) }
                        }
                    }
                }
            }

            // Planting section
            @if seed.sow_depth.is_some() || seed.plant_spacing.is_some() || seed.germination_info.is_some() || seed.planting_instructions.is_some() {
                section.detail-section {
                    h2 { "Planting" }
                    dl.info-list {
                        @if let Some(ref depth) = seed.sow_depth {
                            dt { "Sow Depth" }
                            dd { (depth) }
                        }
                        @if let Some(ref spacing) = seed.plant_spacing {
                            dt { "Plant Spacing" }
                            dd { (spacing) }
                        }
                        @if let Some(ref germ) = seed.germination_info {
                            dt { "Germination" }
                            dd { (germ) }
                        }
                        @if let Some(ref planting) = seed.planting_instructions {
                            dt { "Instructions" }
                            dd { (planting) }
                        }
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

            // Collapsible original scraped text
            @if seed.growing_instructions.is_some() || seed.planting_instructions.is_some() || seed.harvest_instructions.is_some() {
                details.original-text {
                    summary { "Original Scraped Text" }
                    div.original-text-content {
                        @if let Some(ref growing) = seed.growing_instructions {
                            div.text-block {
                                h3 { "Growing Instructions" }
                                p { (growing) }
                            }
                        }
                        @if let Some(ref planting) = seed.planting_instructions {
                            div.text-block {
                                h3 { "Planting Instructions" }
                                p { (planting) }
                            }
                        }
                        @if let Some(ref harvest) = seed.harvest_instructions {
                            div.text-block {
                                h3 { "Harvest Instructions" }
                                p { (harvest) }
                            }
                        }
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
