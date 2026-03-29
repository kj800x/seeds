use maud::{html, Markup};
use std::collections::{HashMap, HashSet};

use crate::db::models::Seed;
use crate::schedule::SowingStatus;
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

/// Render the seed list results wrapper. Used by both the full home page and the search endpoint.
/// Contains an optional error banner + the seed `<ul>`. HTMX swaps this entire div.
pub fn seed_list_fragment(
    seeds: &[Seed],
    newest_purchases: &HashMap<i64, i64>,
    purchase_counts: &HashMap<i64, i64>,
    planned_seeds: &HashSet<i64>,
    sowing_statuses: &HashMap<i64, SowingStatus>,
    error: Option<&str>,
) -> Markup {
    html! {
        div.seed-list-results {
        @if let Some(msg) = error {
            div.search-error { (msg) }
        }
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
                                @if let Some(status) = sowing_statuses.get(&seed.id) {
                                    @let css_class = if status.days_relative == 0 {
                                        "sowing-status sowing-now"
                                    } else if status.days_relative > 0 {
                                        "sowing-status sowing-past"
                                    } else {
                                        "sowing-status"
                                    };
                                    @let timing_text = if status.days_relative < 0 {
                                        format!("in {} days", -status.days_relative)
                                    } else if status.days_relative == 0 {
                                        "now".to_string()
                                    } else {
                                        format!("{} days ago", status.days_relative)
                                    };
                                    span class=(css_class) { (status.method) " \u{2014} " (timing_text) }
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
                            }
                        }
                        (plan_toggle_button(seed.id, in_plan))
                    }
                }
            }
        }
        } // .seed-list-results
    }
}

pub fn home_page(
    seeds: &[Seed],
    newest_purchases: &HashMap<i64, i64>,
    purchase_counts: &HashMap<i64, i64>,
    planned_seeds: &HashSet<i64>,
    sowing_statuses: &HashMap<i64, SowingStatus>,
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
                div.search-wrapper {
                    input.seed-search type="text" name="q"
                          placeholder="Search seeds\u{2026}"
                          hx-get="/search"
                          hx-trigger="input changed delay:300ms, search"
                          hx-target=".seed-list-results"
                          hx-swap="outerHTML"
                          autocomplete="off";
                    div.search-help-toggle tabindex="0" { "?" }
                    div.search-help {
                        p { "Type plain text to search by title, or start with " code { "(" } " for s-expression queries." }
                        h4 { "Logical" }
                        pre { "(and EXPR EXPR ...)\n(or EXPR EXPR ...)\n(not EXPR)" }
                        h4 { "Fields" }
                        pre { "(category \"Vegetables\")\n(subcategory \"Tomato\")\n(title \"cherry\")" }
                        h4 { "Flags" }
                        pre { "(organic)\n(heirloom)" }
                        h4 { "Plan" }
                        pre { "(in-plan)\n(plan \"indoor\")" }
                        h4 { "Timing" }
                        pre { "(start now)\n(start (before \"March 30\"))\n(sow (after now))\n(transplant (before \"2 weeks from now\"))" }
                        h4 { "Viability" }
                        pre { "(viable)\n(viability (above 50))" }
                        h4 { "Examples" }
                        pre { "(and (category \"Vegetables\") (in-plan))\n(or (category \"Flower\") (category \"Herb\"))\n(and (start (before now)) (viable))" }
                    }
                }
            }
            @if seeds.is_empty() {
                div.empty-state {
                    p { "No seeds yet." }
                    p.hint { "Paste a Botanical Interests product URL above to get started." }
                }
            } @else {
                (seed_list_fragment(seeds, newest_purchases, purchase_counts, planned_seeds, sowing_statuses, None))
            }
        }
    };

    layout("Seeds", content)
}
