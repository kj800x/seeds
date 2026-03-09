use std::collections::HashMap;

use chrono::{Datelike, Local, NaiveDate};
use maud::{html, Markup};

use crate::db::models::Seed;
use crate::schedule::{ActionType, PlantingAction, PlantingTiming};
use super::layout::layout;

/// Timeline spans March 1 through September 30.
fn timeline_start(year: i32) -> NaiveDate {
    NaiveDate::from_ymd_opt(year, 3, 1).unwrap()
}

fn timeline_end(year: i32) -> NaiveDate {
    NaiveDate::from_ymd_opt(year, 9, 30).unwrap()
}

/// Convert a date to a percentage position within the timeline.
fn date_to_percent(date: NaiveDate, year: i32) -> f64 {
    let start = timeline_start(year);
    let end = timeline_end(year);
    let total_days = (end - start).num_days() as f64;
    let offset = (date - start).num_days() as f64;
    (offset / total_days * 100.0).clamp(0.0, 100.0)
}

fn action_type_css_class(action_type: &ActionType) -> &'static str {
    match action_type {
        ActionType::StartIndoors => "start-indoors",
        ActionType::TransplantOutdoors => "transplant",
        ActionType::DirectSow => "direct-sow",
    }
}

fn action_type_label(action_type: &ActionType) -> &'static str {
    match action_type {
        ActionType::StartIndoors => "Start indoors",
        ActionType::TransplantOutdoors => "Transplant outdoors",
        ActionType::DirectSow => "Direct sow",
    }
}

/// Format a date as "Mon DD" (e.g., "Mar 29").
fn format_date(date: &NaiveDate) -> String {
    date.format("%b %e").to_string()
}

/// Render the full schedule page with action list and timeline views.
pub fn schedule_page_template(
    actions: &[PlantingAction],
    manual_seeds: &[&Seed],
    seeds_with_timing: &[(Seed, PlantingTiming)],
    year: i32,
) -> Markup {
    let content = html! {
        // Section A: Action List
        section.schedule-section {
            h2 { "Planting Schedule " (year) }

            @if seeds_with_timing.is_empty() {
                div.empty-state {
                    p { "No seeds planned yet." }
                    p.hint { "Go to " a href="/" { "Seeds" } " to add some to your plan." }
                }
            } @else if actions.is_empty() && manual_seeds.is_empty() {
                div.empty-state {
                    p { "No computable planting dates for your planned seeds." }
                    p.hint { "Check the packet instructions for timing details." }
                }
            } @else {
                // Group actions by month
                (render_action_list(actions, year))

                // Manual review section
                @if !manual_seeds.is_empty() {
                    (render_manual_review(manual_seeds))
                }
            }
        }

        // Section B: Visual Timeline
        @if !seeds_with_timing.is_empty() {
            section.schedule-section {
                h2 { "Season Timeline" }
                (render_timeline(actions, seeds_with_timing, year))
            }
        }
    };

    layout("Schedule", content)
}

/// Render the action list grouped by month.
fn render_action_list(actions: &[PlantingAction], _year: i32) -> Markup {
    // Group by month
    let mut month_groups: Vec<(String, Vec<&PlantingAction>)> = Vec::new();
    let mut current_month = String::new();

    for action in actions {
        let month = action.date.format("%B").to_string();
        if month != current_month {
            current_month = month.clone();
            month_groups.push((month, Vec::new()));
        }
        month_groups.last_mut().unwrap().1.push(action);
    }

    html! {
        div.schedule-actions {
            @for (month, group) in &month_groups {
                div.month-header { (month) }
                @for action in group {
                    div.action-row {
                        span.action-date { (format_date(&action.date)) }
                        span.action-type class=(action_type_css_class(&action.action_type)) {
                            (action_type_label(&action.action_type))
                        }
                        span.action-seed {
                            a href=(format!("/seeds/{}", action.seed_id)) {
                                (action.seed_title)
                            }
                        }
                        span.action-notes { (action.notes) }
                    }
                }
            }
        }
    }
}

/// Render the manual review section for seeds with unparseable timing.
fn render_manual_review(manual_seeds: &[&Seed]) -> Markup {
    html! {
        div.manual-review {
            h3 { "Manual Review Needed" }
            p.manual-review-desc { "These seeds have planting instructions that could not be automatically parsed:" }
            @for seed in manual_seeds {
                div.manual-seed-item {
                    a href=(format!("/seeds/{}", seed.id)) { (seed.title) }
                    @if let Some(ref instructions) = seed.planting_instructions {
                        p.manual-seed-instructions { (instructions) }
                    } @else {
                        p.manual-seed-instructions { em { "No planting instructions available" } }
                    }
                }
            }
        }
    }
}

/// Render the CSS Grid timeline.
fn render_timeline(
    actions: &[PlantingAction],
    seeds_with_timing: &[(Seed, PlantingTiming)],
    year: i32,
) -> Markup {
    let months = ["Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep"];

    // Group actions by seed_id for timeline bars
    let mut seed_actions: HashMap<i64, Vec<&PlantingAction>> = HashMap::new();
    for action in actions {
        seed_actions.entry(action.seed_id).or_default().push(action);
    }

    // Today line position
    let today = Local::now().date_naive();
    let t_start = timeline_start(year);
    let t_end = timeline_end(year);
    let show_today = today >= t_start && today <= t_end;
    let today_pct = if show_today { date_to_percent(today, year) } else { 0.0 };

    html! {
        div.timeline {
            // Header row: empty cell + month labels
            div.timeline-seed-name {}
            div.timeline-header {
                @for m in &months {
                    span { (m) }
                }
            }

            // One row per seed
            @for (seed, _timing) in seeds_with_timing {
                div.timeline-row {
                    div.timeline-seed-name { (seed.title) }
                    div.timeline-bars {
                        @if let Some(seed_acts) = seed_actions.get(&seed.id) {
                            @for action in seed_acts {
                                @let left = date_to_percent(action.date, year);
                                // Each action bar is a point marker with small width
                                @let bar_class = format!("timeline-bar {}", action_type_css_class(&action.action_type));
                                div class=(bar_class)
                                    style=(format!("left: {:.1}%; width: 8px;", left))
                                    title=(format!("{}: {} - {}", action_type_label(&action.action_type), format_date(&action.date), action.seed_title)) {}
                            }
                            // If seed has both indoor start and transplant/sow, draw a connecting bar
                            (render_period_bars(seed_acts, year))
                        } @else {
                            // No parseable timing -- show gray "?" bar
                            div.timeline-bar.manual style="left: 0%; width: 100%;"
                                title="See packet instructions" {
                                span.timeline-bar-label { "?" }
                            }
                        }

                        // Today marker
                        @if show_today {
                            div.timeline-today style=(format!("left: {:.1}%;", today_pct)) {}
                        }
                    }
                }
            }
        }
    }
}

/// Render period bars connecting related actions for a seed (e.g., indoor start to transplant).
fn render_period_bars(actions: &[&PlantingAction], year: i32) -> Markup {
    // Find indoor start and transplant/sow dates to draw period bars
    let indoor = actions.iter().find(|a| a.action_type == ActionType::StartIndoors);
    let transplant = actions.iter().find(|a| a.action_type == ActionType::TransplantOutdoors);
    let direct_sow = actions.iter().find(|a| a.action_type == ActionType::DirectSow);

    html! {
        // Indoor period: from start indoors to transplant date
        @if let (Some(start), Some(end)) = (indoor, transplant) {
            @let left = date_to_percent(start.date, year);
            @let right = date_to_percent(end.date, year);
            @let width = right - left;
            @if width > 0.0 {
                div.timeline-bar.start-indoors.period-bar
                    style=(format!("left: {:.1}%; width: {:.1}%;", left, width))
                    title=(format!("Indoor period: {} to {}", format_date(&start.date), format_date(&end.date))) {}
            }
        }

        // Outdoor period: from transplant to ~8 weeks after (approximate growing season)
        @if let Some(tp) = transplant {
            @let left = date_to_percent(tp.date, year);
            // Show a green bar from transplant extending ~6 weeks
            @let end_date = tp.date + chrono::Duration::weeks(6);
            @let right = date_to_percent(end_date, year);
            @let width = right - left;
            @if width > 0.0 {
                div.timeline-bar.transplant.period-bar
                    style=(format!("left: {:.1}%; width: {:.1}%;", left, width))
                    title=(format!("Outdoor growing from {}", format_date(&tp.date))) {}
            }
        }

        // Direct sow period
        @if let Some(ds) = direct_sow {
            @let left = date_to_percent(ds.date, year);
            @let end_date = ds.date + chrono::Duration::weeks(6);
            @let right = date_to_percent(end_date, year);
            @let width = right - left;
            @if width > 0.0 {
                div.timeline-bar.direct-sow.period-bar
                    style=(format!("left: {:.1}%; width: {:.1}%;", left, width))
                    title=(format!("Direct sow growing from {}", format_date(&ds.date))) {}
            }
        }
    }
}
