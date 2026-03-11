use maud::{html, Markup};
use std::collections::{HashMap, HashSet};

use crate::db::models::Seed;
use crate::viability::estimate_viability;
use super::layout::layout;

/// Render a plan toggle button for a seed. Used in both the seed list and the toggle POST response.
pub fn plan_toggle_button(seed_id: i64, in_plan: bool) -> Markup {
    let label = if in_plan { "In Plan" } else { "Add to Plan" };
    let class = if in_plan { "btn-plan-toggle active" } else { "btn-plan-toggle" };
    html! {
        button class=(class)
               hx-post=(format!("/plan/toggle/{}", seed_id))
               hx-swap="outerHTML"
               onclick="event.stopPropagation(); event.preventDefault();"
        {
            (label)
        }
    }
}

pub fn home_page(
    seeds: &[Seed],
    newest_purchases: &HashMap<i64, i64>,
    purchase_counts: &HashMap<i64, i64>,
    planned_seeds: &HashSet<i64>,
) -> Markup {
    let content = html! {
        section.add-seed {
            h2 { "Add a Seed" }
            form hx-post="/seeds/add" hx-indicator=".spinner" hx-target=".form-result" {
                input type="url" name="url"
                      placeholder="Paste a Botanical Interests product URL..."
                      required;
                button type="submit" { "Add Seed" }
                span.spinner { "Importing seed data\u{2026}" }
            }
            div.form-result {}
        }

        section.seed-list-section {
            h2 { "Your Seeds" }
            @if !seeds.is_empty() {
                input.seed-search type="text" placeholder="Search seeds\u{2026}"
                      oninput="let q=this.value.toLowerCase();document.querySelectorAll('.seed-item').forEach(el=>{let t=el.querySelector('.seed-row-title').textContent.toLowerCase();el.style.display=t.includes(q)?'':'none'})";
            }
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
                        @let in_plan = planned_seeds.contains(&seed.id);
                        li.seed-item {
                            a.seed-row href=(format!("/seeds/{}", seed.id)) {
                                div.seed-row-main {
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
                                            span class=(format!("viability {}", est.color_tier())) { (est.percentage) "% viable" }
                                        } @else if newest_year.is_none() {
                                            span.viability-prompt { "Add purchase to see viability" }
                                        }
                                    }
                                }
                                (plan_toggle_button(seed.id, in_plan))
                            }
                        }
                    }
                }
            }
        }
    };

    layout("Seeds", content)
}
