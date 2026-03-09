use axum::extract::State;
use maud::Markup;
use std::collections::HashSet;

use chrono::Datelike;

use crate::db::models::AppState;
use crate::db::queries;
use crate::error::AppError;
use crate::schedule::{generate_schedule, parse_planting_timing, PlantingTiming};
use crate::db::models::Seed;
use crate::templates;

pub async fn schedule_page(
    State(state): State<AppState>,
) -> Result<Markup, AppError> {
    let current_year = chrono::Local::now().year() as i32;
    let planned_seeds = queries::list_planned_seeds(&state.db, current_year as i64).await?;

    // Parse timing for each seed
    let seeds_with_timing: Vec<(Seed, PlantingTiming)> = planned_seeds.iter().map(|seed| {
        let timing = seed.planting_instructions.as_deref()
            .map(parse_planting_timing)
            .unwrap_or_default();
        (seed.clone(), timing)
    }).collect();

    let actions = generate_schedule(&seeds_with_timing, current_year);

    // Identify seeds with no actions (unparseable timing)
    let seeds_with_actions: HashSet<i64> = actions.iter().map(|a| a.seed_id).collect();
    let manual_seeds: Vec<&Seed> = planned_seeds.iter()
        .filter(|s| !seeds_with_actions.contains(&s.id))
        .collect();

    Ok(templates::schedule::schedule_page_template(
        &actions, &manual_seeds, &seeds_with_timing, current_year
    ))
}
