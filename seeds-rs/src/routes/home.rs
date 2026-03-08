use axum::extract::State;
use maud::Markup;
use std::collections::HashMap;

use crate::db::models::AppState;
use crate::db::queries;
use crate::error::AppError;
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
    Ok(home_page(&seeds, &newest_purchases, &purchase_counts))
}
