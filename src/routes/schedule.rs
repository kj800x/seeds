use axum::extract::State;
use maud::Markup;
use std::collections::HashSet;

use chrono::Datelike;

use crate::db::models::AppState;
use crate::db::queries;
use crate::error::AppError;
use crate::schedule::{generate_schedule_with_methods, parse_planting_timing_from_fields, PlantingTiming, StartMethod};
use crate::db::models::Seed;
use crate::templates;

fn seed_to_timing(seed: &Seed) -> PlantingTiming {
    parse_planting_timing_from_fields(
        seed.when_to_sow_outside.as_deref(),
        seed.when_to_start_inside.as_deref(),
    )
}

fn build_seeds_with_timing(planned: &[(Seed, Option<String>)]) -> Vec<(Seed, PlantingTiming, Option<StartMethod>)> {
    planned.iter().map(|(seed, method)| {
        let timing = seed_to_timing(seed);
        let start_method = StartMethod::from_str_opt(method.as_deref());
        (seed.clone(), timing, start_method)
    }).collect()
}

/// GET /schedule - Timeline page
pub async fn schedule_page(
    State(state): State<AppState>,
) -> Result<Markup, AppError> {
    let current_year = chrono::Local::now().year();
    let planned_with_methods = queries::list_planned_seeds_with_method(&state.db, current_year as i64).await?;
    let seeds_with_timing = build_seeds_with_timing(&planned_with_methods);

    Ok(templates::schedule::timeline_page_template(
        &seeds_with_timing, current_year
    ))
}

/// GET /schedule/list - Action list page
pub async fn schedule_list(
    State(state): State<AppState>,
) -> Result<Markup, AppError> {
    let current_year = chrono::Local::now().year();
    let planned_with_methods = queries::list_planned_seeds_with_method(&state.db, current_year as i64).await?;
    let seeds_with_timing = build_seeds_with_timing(&planned_with_methods);

    let actions = generate_schedule_with_methods(&seeds_with_timing, current_year);

    let seeds_with_actions: HashSet<i64> = actions.iter().map(|a| a.seed_id).collect();
    let all_seeds: Vec<&Seed> = planned_with_methods.iter().map(|(s, _)| s).collect();
    let manual_seeds: Vec<&Seed> = all_seeds.iter()
        .filter(|s| !seeds_with_actions.contains(&s.id))
        .copied()
        .collect();

    Ok(templates::schedule::schedule_list_template(
        &actions, &manual_seeds, &seeds_with_timing, current_year
    ))
}

/// GET /schedule/week - This week page
pub async fn this_week(
    State(state): State<AppState>,
) -> Result<Markup, AppError> {
    let current_year = chrono::Local::now().year();
    let today = chrono::Local::now().date_naive();
    let window_end = today + chrono::Duration::days(14);

    let weekday = today.weekday().num_days_from_monday();
    let start_of_week = today - chrono::Duration::days(weekday as i64);

    let planned_with_methods = queries::list_planned_seeds_with_method(&state.db, current_year as i64).await?;
    let seeds_with_timing = build_seeds_with_timing(&planned_with_methods);

    let all_actions = generate_schedule_with_methods(&seeds_with_timing, current_year);

    let filtered: Vec<_> = all_actions.iter()
        .filter(|a| {
            (a.date >= today && a.date <= window_end)
                || (a.date < today && a.date >= start_of_week)
        })
        .cloned()
        .collect();

    let next_action = all_actions.iter()
        .find(|a| a.date > window_end);

    Ok(templates::schedule::this_week_template(
        &filtered, next_action, current_year
    ))
}
