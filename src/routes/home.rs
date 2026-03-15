use axum::extract::State;
use chrono::{Datelike, Local};
use maud::Markup;
use std::collections::{HashMap, HashSet};

use crate::db::models::AppState;
use crate::db::queries;
use crate::error::AppError;
use crate::schedule::{SowingStatus, StartMethod, compute_sowing_status};
use crate::templates::home::home_page;

pub async fn home(State(state): State<AppState>) -> Result<Markup, AppError> {
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
    let plan_methods: HashMap<i64, Option<String>> = queries::list_season_plans(&state.db, current_year as i64)
        .await?
        .into_iter()
        .map(|p| (p.seed_id, p.start_method))
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
    Ok(home_page(&seeds, &newest_purchases, &purchase_counts, &planned_seeds, &sowing_statuses))
}
