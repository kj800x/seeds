use axum::extract::{Query, State};
use chrono::{Datelike, Local, NaiveDate};
use maud::Markup;
use std::collections::{HashMap, HashSet};

use crate::db::models::{AppState, Seed};
use crate::db::queries;
use crate::error::AppError;
use crate::schedule::{SowingStatus, StartMethod, compute_sowing_status};
use crate::search::{self, SearchQuery, SeedContext};
use crate::templates::home::{home_page, seed_list_fragment};

struct SeedData {
    seeds: Vec<Seed>,
    newest_purchases: HashMap<i64, i64>,
    purchase_counts: HashMap<i64, i64>,
    planned_seeds: HashSet<i64>,
    skipped_seeds: HashSet<i64>,
    plan_methods: HashMap<i64, Option<String>>,
    sowing_statuses: HashMap<i64, SowingStatus>,
    thumbnails: HashMap<i64, String>,
    today: NaiveDate,
    current_year: i32,
}

async fn load_seed_data(state: &AppState) -> Result<SeedData, AppError> {
    let seeds = queries::list_seeds(&state.db).await?;
    let newest_purchases: HashMap<i64, i64> = queries::newest_purchase_per_seed(&state.db)
        .await?
        .into_iter()
        .collect();
    let purchase_counts: HashMap<i64, i64> = queries::purchase_count_per_seed(&state.db)
        .await?
        .into_iter()
        .collect();
    let today = Local::now().date_naive();
    let current_year = today.year();
    let planned_seeds: HashSet<i64> = queries::planned_seed_ids(&state.db, current_year as i64)
        .await?
        .into_iter()
        .collect();
    let skipped_seeds: HashSet<i64> = queries::skipped_seed_ids(&state.db, current_year as i64)
        .await?
        .into_iter()
        .collect();
    let plan_methods: HashMap<i64, Option<String>> = queries::list_season_plans(&state.db, current_year as i64)
        .await?
        .into_iter()
        .map(|p| (p.seed_id, p.start_method))
        .collect();
    let thumbnails: HashMap<i64, String> = queries::first_image_per_seed(&state.db)
        .await?
        .into_iter()
        .collect();
    let sowing_statuses: HashMap<i64, SowingStatus> = seeds.iter()
        .filter_map(|seed| {
            let method_override = plan_methods.get(&seed.id)
                .and_then(|m| StartMethod::from_str_opt(m.as_deref()));
            compute_sowing_status(seed, today, current_year, method_override)
                .map(|status| (seed.id, status))
        })
        .collect();
    let mut seeds = seeds;
    seeds.sort_by_key(|s| {
        sowing_statuses.get(&s.id)
            .map(|st| st.start_date)
            .unwrap_or(chrono::NaiveDate::MAX)
    });
    Ok(SeedData {
        seeds,
        newest_purchases,
        purchase_counts,
        planned_seeds,
        skipped_seeds,
        plan_methods,
        sowing_statuses,
        thumbnails,
        today,
        current_year,
    })
}

pub async fn home(State(state): State<AppState>) -> Result<Markup, AppError> {
    let data = load_seed_data(&state).await?;
    // Hide skipped seeds from the default listing
    let visible_seeds: Vec<Seed> = data.seeds.iter()
        .filter(|s| !data.skipped_seeds.contains(&s.id))
        .cloned()
        .collect();
    Ok(home_page(&visible_seeds, &data.newest_purchases, &data.purchase_counts, &data.planned_seeds, &data.skipped_seeds, &data.sowing_statuses, &data.thumbnails))
}

#[derive(serde::Deserialize)]
pub struct SearchParams {
    #[serde(default)]
    q: String,
}

pub async fn search(
    State(state): State<AppState>,
    Query(params): Query<SearchParams>,
) -> Result<Markup, AppError> {
    let data = load_seed_data(&state).await?;

    let query = search::parse_query(&params.q);

    // Determine whether to include skipped seeds in results
    let include_skipped = match &query {
        Ok(SearchQuery::SExp(filter)) => filter.references_skipped(),
        _ => false,
    };

    let (filtered_seeds, error_msg) = match query {
        Ok(SearchQuery::Plaintext(ref q)) if q.is_empty() => {
            let visible: Vec<Seed> = data.seeds.iter()
                .filter(|s| !data.skipped_seeds.contains(&s.id))
                .cloned()
                .collect();
            (visible, None)
        }
        Ok(SearchQuery::Plaintext(ref q)) => {
            let filtered: Vec<Seed> = data.seeds.iter()
                .filter(|s| !data.skipped_seeds.contains(&s.id) && s.title.to_lowercase().contains(q))
                .cloned()
                .collect();
            (filtered, None)
        }
        Ok(SearchQuery::SExp(ref filter)) => {
            let filtered: Vec<Seed> = data.seeds.iter()
                .filter(|seed| {
                    // Hide skipped unless query explicitly references them
                    if !include_skipped && data.skipped_seeds.contains(&seed.id) {
                        return false;
                    }
                    let ctx = SeedContext {
                        seed,
                        in_plan: data.planned_seeds.contains(&seed.id),
                        is_skipped: data.skipped_seeds.contains(&seed.id),
                        plan_method: data.plan_methods.get(&seed.id)
                            .and_then(|m| m.as_deref()),
                        sowing_status: data.sowing_statuses.get(&seed.id),
                        newest_purchase_year: data.newest_purchases.get(&seed.id).copied(),
                        today: data.today,
                        current_year: data.current_year,
                    };
                    search::matches(filter, &ctx)
                })
                .cloned()
                .collect();
            (filtered, None)
        }
        Err(e) => {
            // On parse error, show all seeds (minus skipped) with a warning
            let visible: Vec<Seed> = data.seeds.iter()
                .filter(|s| !data.skipped_seeds.contains(&s.id))
                .cloned()
                .collect();
            (visible, Some(e))
        }
    };

    Ok(seed_list_fragment(
        &filtered_seeds,
        &data.newest_purchases,
        &data.purchase_counts,
        &data.planned_seeds,
        &data.skipped_seeds,
        &data.sowing_statuses,
        &data.thumbnails,
        error_msg.as_deref(),
    ))
}
