use axum::extract::State;
use maud::Markup;

use crate::db::models::AppState;
use crate::error::AppError;
use crate::templates::settings;

pub async fn settings_page() -> Markup {
    settings::settings_page()
}

pub async fn reset_all_data(State(state): State<AppState>) -> Result<Markup, AppError> {
    // Delete in dependency order
    sqlx::query("DELETE FROM season_plan_events")
        .execute(&state.db)
        .await?;
    sqlx::query("DELETE FROM season_plans")
        .execute(&state.db)
        .await?;
    sqlx::query("DELETE FROM seed_purchases")
        .execute(&state.db)
        .await?;
    sqlx::query("DELETE FROM seed_images")
        .execute(&state.db)
        .await?;
    sqlx::query("DELETE FROM seeds")
        .execute(&state.db)
        .await?;

    // Clean up image files
    let images_dir = state.data_dir.join("images");
    if images_dir.exists()
        && let Ok(entries) = std::fs::read_dir(&images_dir)
    {
        for entry in entries.flatten() {
            let _ = std::fs::remove_file(entry.path());
        }
    }

    Ok(settings::reset_success())
}
