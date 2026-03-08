use maud::{html, Markup, PreEscaped};

use crate::db::models::{Seed, SeedImage};
use crate::viability::estimate_viability;
use super::layout::layout;

/// Render the seed info section (inventory info + viability).
/// Used by both the full page template and the HTMX fragment handlers.
pub fn seed_info_section(seed: &Seed) -> Markup {
    let viability = estimate_viability(
        seed.subcategory.as_deref(),
        seed.category.as_deref(),
        seed.purchase_year,
    );

    html! {
        div #seed-info {
            section.detail-section {
                h2 { "Inventory Info" }
                dl.info-list {
                    dt { "Purchase Year" }
                    dd {
                        @if let Some(py) = seed.purchase_year {
                            (py)
                        } @else {
                            span.viability-prompt { "Not set" }
                        }
                    }
                    @if let Some(ref notes) = seed.notes {
                        dt { "Notes" }
                        dd { (notes) }
                    }
                    dt { "Viability" }
                    dd {
                        @if let Some(ref est) = viability {
                            span.viability-display { (est.percentage) "%" }
                            " "
                            span.viability-detail {
                                "(" (est.age_years) " of " (est.max_years)
                                " year lifespan, matched on " (est.species_key) ")"
                            }
                        } @else if seed.purchase_year.is_none() {
                            span.viability-prompt { "Set purchase year to see viability estimate" }
                        } @else {
                            span.viability-prompt { "Viability unavailable (no species data)" }
                        }
                    }
                }
                button.btn.btn-edit
                       hx-get=(format!("/seeds/{}/edit", seed.id))
                       hx-target="#seed-info"
                       hx-swap="outerHTML" { "Edit" }
            }
        }
    }
}

pub fn seed_detail_page(seed: &Seed, images: &[SeedImage]) -> Markup {
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

            // Inventory Info section (with viability)
            (seed_info_section(seed))

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
