use maud::{html, Markup};

use crate::db::models::Seed;
use crate::viability::estimate_viability;
use super::layout::layout;

pub fn home_page(seeds: &[Seed]) -> Markup {
    let content = html! {
        section.add-seed {
            h2 { "Add a Seed" }
            form hx-post="/seeds/add" hx-indicator=".spinner" hx-target=".form-result" {
                input type="url" name="url"
                      placeholder="Paste a Botanical Interests product URL..."
                      required;
                input type="number" name="purchase_year"
                      placeholder="Purchase year (e.g. 2025)"
                      min="2000" max="2030"
                      class="purchase-year-input";
                button type="submit" { "Add Seed" }
                span.spinner { "Adding seed..." }
            }
            div.form-result {}
        }

        section.seed-list-section {
            h2 { "Your Seeds" }
            @if seeds.is_empty() {
                div.empty-state {
                    p { "No seeds yet." }
                    p.hint { "Paste a Botanical Interests product URL above to get started." }
                }
            } @else {
                ul.seed-list {
                    @for seed in seeds {
                        li {
                            a.seed-row href=(format!("/seeds/{}", seed.id)) {
                                div.seed-row-title { (seed.title) }
                                div.seed-row-meta {
                                    @if let Some(ref cat) = seed.category {
                                        span { (cat) }
                                    }
                                    @if let Some(ref subcat) = seed.subcategory {
                                        span { (subcat) }
                                    }
                                    @if let Some(ref dtm) = seed.days_to_maturity {
                                        span { (dtm) " days" }
                                    }
                                    @if let Some(py) = seed.purchase_year {
                                        span { "Purchased " (py) }
                                    }
                                    @let viability = estimate_viability(
                                        seed.subcategory.as_deref(),
                                        seed.category.as_deref(),
                                        seed.purchase_year,
                                    );
                                    @if let Some(ref est) = viability {
                                        span.viability { (est.percentage) "% viable" }
                                    } @else if seed.purchase_year.is_none() {
                                        span.viability-prompt { "Set year for viability" }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    };

    layout("Seeds", content)
}
