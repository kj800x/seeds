use chrono::{Local, NaiveDate};
use maud::{html, Markup};

use crate::db::models::{Seed, SeasonPlanEvent};
use crate::schedule::{ActionType, PlantingAction, PlantingTiming, compute_seed_timeline, compute_indoor_timeline, compute_outdoor_timeline, compute_timeline_for_method, StartMethod, SeedTimeline};
use crate::schedule::calculator::{PhaseType, last_frost_date};
use super::layout::layout_with_nav;

/// Timeline spans February 1 through October 31.
fn timeline_start(year: i32) -> NaiveDate {
    NaiveDate::from_ymd_opt(year, 2, 1).unwrap()
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

/// Render the tab navigation for schedule pages.
fn schedule_tabs(active_tab: &str) -> Markup {
    html! {
        div.schedule-tabs {
            a.tab href="/schedule"
                class=@if active_tab == "timeline" { "active" } { "Timeline" }
            a.tab href="/schedule/list"
                class=@if active_tab == "list" { "active" } { "Schedule" }
            a.tab href="/schedule/week"
                class=@if active_tab == "week" { "active" } { "This Week" }
        }
    }
}

/// Render the Timeline page (visual season overview).
pub fn timeline_page_template(
    seeds_with_timing: &[(Seed, PlantingTiming, Option<StartMethod>)],
    year: i32,
) -> Markup {
    let content = html! {
        (schedule_tabs("timeline"))

        div.schedule-content {
            section.schedule-section {
                h2 { "Season Timeline " (year) }

                @if seeds_with_timing.is_empty() {
                    div.empty-state {
                        p { "No seeds planned yet." }
                        p.hint { "Go to " a href="/" { "Seeds" } " to add some to your plan." }
                    }
                } @else {
                    (render_timeline_legend())
                    (render_timeline(seeds_with_timing, year))
                }
            }
        }
    };

    layout_with_nav("Timeline", "schedule", content)
}

/// Render the Schedule page (action list grouped by month).
pub fn schedule_list_template(
    actions: &[PlantingAction],
    manual_seeds: &[&Seed],
    seeds_with_timing: &[(Seed, PlantingTiming, Option<StartMethod>)],
    year: i32,
) -> Markup {
    let content = html! {
        (schedule_tabs("list"))

        div.schedule-content {
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
                    (render_action_list(actions, year))

                    @if !manual_seeds.is_empty() {
                        (render_manual_review(manual_seeds))
                    }
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
        (schedule_tabs("week"))

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
    let months = ["Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct"];

    // Today line position
    let today = Local::now().date_naive();
    let t_start = timeline_start(year);
    let t_end = timeline_end(year);
    let show_today = today >= t_start && today <= t_end;
    let today_pct = if show_today { date_to_percent(today, year) } else { 0.0 };

    // Last frost line
    let frost = last_frost_date(year);
    let frost_pct = date_to_percent(frost, year);

    // Compute timelines and sort by earliest phase start date
    let mut rows: Vec<(&Seed, SeedTimeline)> = seeds_with_timing.iter().map(|(seed, timing, method)| {
        let timeline = if let Some(m) = method {
            compute_timeline_for_method(seed, timing, year, *m)
        } else {
            compute_seed_timeline(seed, timing, year)
        };
        (seed, timeline)
    }).collect();
    rows.sort_by_key(|(_, tl)| tl.phases.first().map(|p| p.start).unwrap_or(NaiveDate::MAX));

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
            @for (seed, timeline) in &rows {
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
pub fn seed_detail_timeline(seed: &Seed, timing: &PlantingTiming, year: i32, in_plan: bool, is_skipped: bool, events: &[SeasonPlanEvent]) -> Markup {
    let timeline = compute_seed_timeline(seed, timing, year);
    let event_refs: Vec<&SeasonPlanEvent> = events.iter().collect();

    if timeline.phases.is_empty() {
        return html! {
            section #timeline-section .detail-section {
                h2 { "Plan " (year) }
                (render_plan_controls(seed, in_plan, is_skipped, false, false, None))
            }
        };
    }

    html! {
        section #timeline-section .detail-section {
            h2 { "Timeline " (year) }
            (render_timeline_legend())
            (render_single_timeline(&timeline, year, &event_refs))
            (render_key_dates(&timeline, year, &event_refs))
            (render_plan_controls(seed, in_plan, is_skipped, false, false, None))
        }
    }
}

/// Render separate indoor and outdoor timelines on the seed detail page,
/// with a "Recommended" badge on the appropriate one.
pub fn seed_detail_dual_timeline(seed: &Seed, timing: &PlantingTiming, year: i32, in_plan: bool, is_skipped: bool, plan_start_method: Option<&str>, events: &[SeasonPlanEvent]) -> Markup {
    let indoor = compute_indoor_timeline(seed, timing, year);
    let outdoor = compute_outdoor_timeline(seed, timing, year);
    let event_refs: Vec<&SeasonPlanEvent> = events.iter().collect();

    if indoor.is_none() && outdoor.is_none() {
        return html! {
            section #timeline-section .detail-section {
                h2 { "Plan " (year) }
                (render_plan_controls(seed, in_plan, is_skipped, false, false, None))
            }
        };
    }

    let has_indoor = indoor.is_some();
    let has_outdoor = outdoor.is_some();

    // Split events by relevance to indoor vs outdoor timelines
    let indoor_events: Vec<&SeasonPlanEvent> = event_refs.iter()
        .filter(|e| e.event_type == "sow_indoor" || e.event_type == "transplant")
        .copied()
        .collect();
    let outdoor_events: Vec<&SeasonPlanEvent> = event_refs.iter()
        .filter(|e| e.event_type == "sow_outdoor")
        .copied()
        .collect();

    // Determine which strategies have sow events logged
    let has_indoor_sow = events.iter().any(|e| e.event_type == "sow_indoor");
    let has_outdoor_sow = events.iter().any(|e| e.event_type == "sow_outdoor");
    let has_any_sow = has_indoor_sow || has_outdoor_sow;

    // Auto-open if: no sow events yet, or this strategy has events
    let indoor_open = !has_any_sow || has_indoor_sow;
    let outdoor_open = !has_any_sow || has_outdoor_sow;

    html! {
        section #timeline-section .detail-section {
            h2 { "Timeline " (year) }
            (render_timeline_legend())

            @if let Some(ref indoor_tl) = indoor {
                details.timeline-method-section open[indoor_open] {
                    summary.timeline-method-header {
                        h3 { "Start Indoors" }
                        @if timing.indoor_start_recommended {
                            span.badge.badge-recommended { "Recommended" }
                        }
                    }
                    (render_single_timeline(indoor_tl, year, &indoor_events))
                    (render_key_dates(indoor_tl, year, &indoor_events))
                }
            }

            @if let Some(ref outdoor_tl) = outdoor {
                details.timeline-method-section open[outdoor_open] {
                    summary.timeline-method-header {
                        h3 { "Start Outdoors" }
                        @if !timing.indoor_start_recommended {
                            span.badge.badge-recommended { "Recommended" }
                        }
                    }
                    (render_single_timeline(outdoor_tl, year, &outdoor_events))
                    (render_key_dates(outdoor_tl, year, &outdoor_events))
                }
            }

            (render_plan_controls(seed, in_plan, is_skipped, has_indoor, has_outdoor, plan_start_method))
        }
    }
}

/// Render plan controls: add-to-plan toggle and optional start method selector.
fn render_plan_controls(seed: &Seed, in_plan: bool, is_skipped: bool, has_indoor: bool, has_outdoor: bool, plan_start_method: Option<&str>) -> Markup {
    let (label, class) = if is_skipped {
        ("Skipped", "btn-plan-toggle skipped")
    } else if in_plan {
        ("In Plan", "btn-plan-toggle active")
    } else {
        ("Add to Plan", "btn-plan-toggle")
    };
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

/// Map event_type to ActionType for matching against key dates.
fn event_type_to_action(event_type: &str) -> Option<ActionType> {
    match event_type {
        "sow_indoor" => Some(ActionType::StartIndoors),
        "sow_outdoor" => Some(ActionType::DirectSow),
        "transplant" => Some(ActionType::TransplantOutdoors),
        _ => None,
    }
}

fn event_type_marker_label(event_type: &str) -> &str {
    match event_type {
        "sow_indoor" => "Sowed indoors",
        "sow_outdoor" => "Sowed outdoors",
        "transplant" => "Transplanted",
        _ => "",
    }
}

/// Render a single timeline bar (reusable for both combined and split views).
fn render_single_timeline(timeline: &SeedTimeline, year: i32, events: &[&SeasonPlanEvent]) -> Markup {
    let months = ["Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct"];

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

                    // Event markers
                    @for event in events {
                        @if let Ok(date) = NaiveDate::parse_from_str(&event.event_date, "%Y-%m-%d") {
                            @let pct = date_to_percent(date, year);
                            div class=(format!("timeline-event-marker event-marker-{}", event.event_type))
                                style=(format!("left: {:.1}%;", pct))
                                title=(format!("{} on {}", event_type_marker_label(&event.event_type), format_date(&date))) {}
                        }
                    }
                }
            }
        }
    }
}

/// Render the key dates list for a timeline.
fn render_key_dates(timeline: &SeedTimeline, year: i32, events: &[&SeasonPlanEvent]) -> Markup {
    let frost = last_frost_date(year);

    html! {
        div.key-dates {
            h3 { "Key Dates" }
            dl.info-list {
                @for action in &timeline.actions {
                    @let done_event = events.iter().find(|e| event_type_to_action(&e.event_type) == Some(action.action_type.clone()));
                    dt { (action_type_label(&action.action_type)) }
                    dd {
                        @if let Some(evt) = done_event {
                            span.key-date-done {
                                (format_date(&NaiveDate::parse_from_str(&evt.event_date, "%Y-%m-%d").unwrap_or(action.date)))
                            }
                            " "
                            span.key-date-check { "\u{2713}" }
                            @if action.date != NaiveDate::parse_from_str(&evt.event_date, "%Y-%m-%d").unwrap_or(action.date) {
                                " "
                                span.key-date-planned { "planned " (format_date(&action.date)) }
                            }
                        } @else {
                            (format_date(&action.date)) " (" (action.notes) ")"
                        }
                    }
                }
                dt { "Last Frost" }
                dd { (format_date(&frost)) }
            }
        }
    }
}
