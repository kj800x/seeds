use maud::{html, Markup, PreEscaped};

use crate::db::models::{Seed, SeedImage};
use super::layout::layout;

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
        }
    };

    layout(&seed.title, content)
}
