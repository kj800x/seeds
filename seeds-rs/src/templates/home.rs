use maud::{html, Markup};
use std::collections::HashMap;

use crate::db::models::Seed;
use crate::viability::estimate_viability;
use super::layout::layout;

pub fn home_page(
    seeds: &[Seed],
    newest_purchases: &HashMap<i64, i64>,
    purchase_counts: &HashMap<i64, i64>,
) -> Markup {
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
                span.spinner { "Importing seed data\u{2026}" }
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
                        @let newest_year = newest_purchases.get(&seed.id).copied();
                        @let count = purchase_counts.get(&seed.id).copied().unwrap_or(0);
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
                                    @if let Some(year) = newest_year {
                                        span {
                                            @if count > 1 {
                                                (count) " lots (newest " (year) ")"
                                            } @else {
                                                "Purchased " (year)
                                            }
                                        }
                                    }
                                    @let viability = estimate_viability(
                                        seed.subcategory.as_deref(),
                                        seed.category.as_deref(),
                                        newest_year,
                                    );
                                    @if let Some(ref est) = viability {
                                        span.viability { (est.percentage) "% viable" }
                                    } @else if newest_year.is_none() {
                                        span.viability-prompt { "Add purchase to see viability" }
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
