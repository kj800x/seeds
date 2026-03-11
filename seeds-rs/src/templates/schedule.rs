use chrono::{Local, NaiveDate};
use maud::{html, Markup};

use crate::db::models::Seed;
use crate::schedule::{ActionType, PlantingAction, PlantingTiming, compute_seed_timeline, compute_indoor_timeline, compute_outdoor_timeline, compute_timeline_for_method, StartMethod, SeedTimeline};
use crate::schedule::calculator::{PhaseType, last_frost_date};
use super::home::plan_toggle_button;
use super::layout::layout_with_nav;

/// Timeline spans March 1 through October 31.
fn timeline_start(year: i32) -> NaiveDate {
    NaiveDate::from_ymd_opt(year, 3, 1).unwrap()
}

fn timeline_end(year: i32) -> NaiveDate {
    NaiveDate::from_ymd_opt(year, 10, 31).unwrap()
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

fn phase_type_css_class(phase_type: &PhaseType) -> &'static str {
    match phase_type {
        PhaseType::PlantingWindow => "phase-planting-window",
        PhaseType::IndoorSowing => "phase-indoor-sow",
        PhaseType::IndoorGrowing => "phase-indoor",
        PhaseType::TransplantWindow => "phase-transplant",
        PhaseType::OutdoorGrowing => "phase-outdoor",
        PhaseType::Harvest => "phase-harvest",
    }
}

fn phase_type_label(phase_type: &PhaseType) -> &'static str {
    match phase_type {
        PhaseType::PlantingWindow => "Outdoor sowing",
        PhaseType::IndoorSowing => "Seeding",
        PhaseType::IndoorGrowing => "Indoor growth",
        PhaseType::TransplantWindow => "Transplant",
        PhaseType::OutdoorGrowing => "Outdoor growth",
        PhaseType::Harvest => "Harvest",
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
    seeds_with_timing: &[(Seed, PlantingTiming, Option<StartMethod>)],
    year: i32,
) -> Markup {
    let content = html! {
        // Tab navigation
        div.schedule-tabs {
            a.tab.active href="/schedule"
                hx-get="/schedule" hx-target=".schedule-content" hx-push-url="true" { "Full Season" }
            a.tab href="/schedule/week"
                hx-get="/schedule/week" hx-target=".schedule-content" hx-push-url="true" { "This Week" }
        }

        div.schedule-content {
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
                    (render_timeline_legend())
                    (render_timeline(seeds_with_timing, year))
                }
            }
        }
    };

    layout_with_nav("Schedule", "schedule", content)
}

/// Render the "This Week" filtered schedule view.
pub fn this_week_template(
    actions: &[PlantingAction],
    next_action: Option<&PlantingAction>,
    year: i32,
) -> Markup {
    let content = html! {
        // Tab navigation
        div.schedule-tabs {
            a.tab href="/schedule"
                hx-get="/schedule" hx-target=".schedule-content" hx-push-url="true" { "Full Season" }
            a.tab.active href="/schedule/week"
                hx-get="/schedule/week" hx-target=".schedule-content" hx-push-url="true" { "This Week" }
        }

        div.schedule-content {
            section.schedule-section {
                h2 { "This Week" }

                @if actions.is_empty() {
                    div.empty-state {
                        p { "No actions this week." }
                        @if let Some(next) = next_action {
                            p.hint { "Next up: " (action_type_label(&next.action_type)) " " (next.seed_title) " on " (format_date(&next.date)) "." }
                        }
                    }
                } @else {
                    (render_action_list(actions, year))
                }
            }
        }
    };

    layout_with_nav("This Week", "schedule", content)
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

/// Render the timeline legend.
fn render_timeline_legend() -> Markup {
    html! {
        div.timeline-legend {
            span.legend-item {
                span.legend-swatch.phase-indoor-sow {}
                "Seeding"
            }
            span.legend-item {
                span.legend-swatch.phase-indoor {}
                "Indoor growth"
            }
            span.legend-item {
                span.legend-swatch.phase-planting-window {}
                "Outdoor sow"
            }
            span.legend-item {
                span.legend-swatch.phase-transplant {}
                "Transplant"
            }
            span.legend-item {
                span.legend-swatch.phase-outdoor {}
                "Outdoor growth"
            }
            span.legend-item {
                span.legend-swatch.phase-harvest {}
                "Harvest"
            }
            span.legend-item {
                span.legend-swatch.legend-frost {}
                "Last frost"
            }
            span.legend-item {
                span.legend-swatch.legend-today {}
                "Today"
            }
        }
    }
}

/// Render the phase-based timeline.
fn render_timeline(
    seeds_with_timing: &[(Seed, PlantingTiming, Option<StartMethod>)],
    year: i32,
) -> Markup {
    let months = ["Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct"];

    // Today line position
    let today = Local::now().date_naive();
    let t_start = timeline_start(year);
    let t_end = timeline_end(year);
    let show_today = today >= t_start && today <= t_end;
    let today_pct = if show_today { date_to_percent(today, year) } else { 0.0 };

    // Last frost line
    let frost = last_frost_date(year);
    let frost_pct = date_to_percent(frost, year);

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
            @for (seed, timing, method) in seeds_with_timing {
                @let timeline = if let Some(m) = method {
                    compute_timeline_for_method(seed, timing, year, *m)
                } else {
                    compute_seed_timeline(seed, timing, year)
                };
                div.timeline-row {
                    div.timeline-seed-name {
                        a href=(format!("/seeds/{}", seed.id)) { (seed.title) }
                    }
                    div.timeline-bars {
                        // Render phase bars
                        @for phase in &timeline.phases {
                            @let left = date_to_percent(phase.start, year);
                            @let right = date_to_percent(phase.end, year);
                            @let width = (right - left).max(0.5);
                            div class=(format!("timeline-phase {}", phase_type_css_class(&phase.phase_type)))
                                style=(format!("left: {:.1}%; width: {:.1}%;", left, width))
                                title=(format!("{}: {} - {}", phase_type_label(&phase.phase_type), format_date(&phase.start), format_date(&phase.end))) {}
                        }

                        // If no phases, show gray "?" bar
                        @if timeline.phases.is_empty() {
                            div.timeline-phase.phase-manual style="left: 0%; width: 100%;"
                                title="See packet instructions" {
                                span.timeline-bar-label { "?" }
                            }
                        }

                        // Last frost marker
                        div.timeline-frost style=(format!("left: {:.1}%;", frost_pct)) {}

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

/// Render a mini timeline for the seed detail page.
pub fn seed_detail_timeline(seed: &Seed, timing: &PlantingTiming, year: i32, in_plan: bool) -> Markup {
    let timeline = compute_seed_timeline(seed, timing, year);

    if timeline.phases.is_empty() {
        return html! {
            section #timeline-section .detail-section {
                h2 { "Plan " (year) }
                (render_plan_controls(seed, in_plan, false, false, None))
            }
        };
    }

    html! {
        section #timeline-section .detail-section {
            h2 { "Timeline " (year) }
            (render_timeline_legend())
            (render_single_timeline(&timeline, year))
            (render_key_dates(&timeline, year))
            (render_plan_controls(seed, in_plan, false, false, None))
        }
    }
}

/// Render separate indoor and outdoor timelines on the seed detail page,
/// with a "Recommended" badge on the appropriate one.
pub fn seed_detail_dual_timeline(seed: &Seed, timing: &PlantingTiming, year: i32, in_plan: bool, plan_start_method: Option<&str>) -> Markup {
    let indoor = compute_indoor_timeline(seed, timing, year);
    let outdoor = compute_outdoor_timeline(seed, timing, year);

    if indoor.is_none() && outdoor.is_none() {
        return html! {
            section #timeline-section .detail-section {
                h2 { "Plan " (year) }
                (render_plan_controls(seed, in_plan, false, false, None))
            }
        };
    }

    let has_indoor = indoor.is_some();
    let has_outdoor = outdoor.is_some();

    html! {
        section #timeline-section .detail-section {
            h2 { "Timeline " (year) }
            (render_timeline_legend())

            @if let Some(ref indoor_tl) = indoor {
                div.timeline-method-section {
                    div.timeline-method-header {
                        h3 { "Start Indoors" }
                        @if timing.indoor_start_recommended {
                            span.badge.badge-recommended { "Recommended" }
                        }
                    }
                    (render_single_timeline(indoor_tl, year))
                    (render_key_dates(indoor_tl, year))
                }
            }

            @if let Some(ref outdoor_tl) = outdoor {
                div.timeline-method-section {
                    div.timeline-method-header {
                        h3 { "Start Outdoors" }
                        @if !timing.indoor_start_recommended {
                            span.badge.badge-recommended { "Recommended" }
                        }
                    }
                    (render_single_timeline(outdoor_tl, year))
                    (render_key_dates(outdoor_tl, year))
                }
            }

            (render_plan_controls(seed, in_plan, has_indoor, has_outdoor, plan_start_method))
        }
    }
}

/// Render plan controls: add-to-plan toggle and optional start method selector.
fn render_plan_controls(seed: &Seed, in_plan: bool, has_indoor: bool, has_outdoor: bool, plan_start_method: Option<&str>) -> Markup {
    let label = if in_plan { "In Plan" } else { "Add to Plan" };
    let class = if in_plan { "btn-plan-toggle active" } else { "btn-plan-toggle" };
    let show_method_selector = in_plan && has_indoor && has_outdoor;
    html! {
        div.plan-controls {
            button class=(class)
                   hx-post=(format!("/plan/toggle/{}?detail=1", seed.id))
                   hx-target="#timeline-section"
                   hx-swap="outerHTML"
            {
                (label)
            }
            @if show_method_selector {
                @if let Some(current_method) = plan_start_method {
                    div.start-method-selector {
                        span.start-method-label { "Start method:" }
                        button class=(if current_method == "indoor" { "btn btn-sm btn-method active" } else { "btn btn-sm btn-method" })
                               hx-post=(format!("/plan/{}/start-method", seed.id))
                               hx-vals=(r#"{"method": "indoor"}"#)
                               hx-target="#timeline-section"
                               hx-swap="outerHTML" {
                            "Indoors"
                        }
                        button class=(if current_method == "outdoor" { "btn btn-sm btn-method active" } else { "btn btn-sm btn-method" })
                               hx-post=(format!("/plan/{}/start-method", seed.id))
                               hx-vals=(r#"{"method": "outdoor"}"#)
                               hx-target="#timeline-section"
                               hx-swap="outerHTML" {
                            "Outdoors"
                        }
                    }
                }
            }
        }
    }
}

/// Render a single timeline bar (reusable for both combined and split views).
fn render_single_timeline(timeline: &SeedTimeline, year: i32) -> Markup {
    let months = ["Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct"];

    let today = Local::now().date_naive();
    let t_start = timeline_start(year);
    let t_end = timeline_end(year);
    let show_today = today >= t_start && today <= t_end;
    let today_pct = if show_today { date_to_percent(today, year) } else { 0.0 };

    let frost = last_frost_date(year);
    let frost_pct = date_to_percent(frost, year);

    html! {
        div.timeline.timeline-single {
            div.timeline-seed-name {}
            div.timeline-header {
                @for m in &months {
                    span { (m) }
                }
            }

            div.timeline-row {
                div.timeline-seed-name {}
                div.timeline-bars {
                    @for phase in &timeline.phases {
                        @let left = date_to_percent(phase.start, year);
                        @let right = date_to_percent(phase.end, year);
                        @let width = (right - left).max(0.5);
                        div class=(format!("timeline-phase {}", phase_type_css_class(&phase.phase_type)))
                            style=(format!("left: {:.1}%; width: {:.1}%;", left, width))
                            title=(format!("{}: {} - {}", phase_type_label(&phase.phase_type), format_date(&phase.start), format_date(&phase.end))) {}
                    }

                    // Last frost marker
                    div.timeline-frost style=(format!("left: {:.1}%;", frost_pct)) {}

                    // Today marker
                    @if show_today {
                        div.timeline-today style=(format!("left: {:.1}%;", today_pct)) {}
                    }
                }
            }
        }
    }
}

/// Render the key dates list for a timeline.
fn render_key_dates(timeline: &SeedTimeline, year: i32) -> Markup {
    let frost = last_frost_date(year);

    html! {
        div.key-dates {
            h3 { "Key Dates" }
            dl.info-list {
                @for action in &timeline.actions {
                    dt { (action_type_label(&action.action_type)) }
                    dd { (format_date(&action.date)) " (" (action.notes) ")" }
                }
                dt { "Last Frost" }
                dd { (format_date(&frost)) }
            }
        }
    }
}
