use axum::extract::State;
use maud::Markup;

use crate::db::models::AppState;
use crate::db::queries;
use crate::error::AppError;
use crate::templates::home::home_page;

pub async fn home(State(state): State<AppState>) -> Result<Markup, AppError> {
    let seeds = queries::list_seeds(&state.db).await?;
    Ok(home_page(&seeds))
}
