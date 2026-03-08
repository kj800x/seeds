use axum::extract::{Form, Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use maud::{html, Markup};
use serde::Deserialize;

use crate::db::models::AppState;
use crate::db::queries;
use crate::error::AppError;
use crate::scraper;
use crate::templates::seed_detail::seed_detail_page;

#[derive(Deserialize)]
pub struct AddSeedInput {
    pub url: String,
}

pub async fn seed_detail(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Markup, AppError> {
    let seed = queries::get_seed(&state.db, id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Seed {} not found", id)))?;

    let images = queries::get_seed_images(&state.db, id).await?;

    Ok(seed_detail_page(&seed, &images))
}

pub async fn add_seed(
    State(state): State<AppState>,
    Form(input): Form<AddSeedInput>,
) -> Response {
    // Validate URL — must be a botanicalinterests.com URL containing /products/
    let url = input.url.trim();
    let host_ok = url.starts_with("https://www.botanicalinterests.com/")
        || url.starts_with("http://www.botanicalinterests.com/")
        || url.starts_with("https://botanicalinterests.com/")
        || url.starts_with("http://botanicalinterests.com/");
    let has_products = url.contains("/products/");

    if !host_ok || !has_products {
        let fragment = html! {
            div.error-message {
                p { "Not a valid Botanical Interests product URL" }
            }
        };
        return (StatusCode::UNPROCESSABLE_ENTITY, fragment).into_response();
    }

    match scraper::scrape_and_save(&state, url).await {
        Ok(seed_id) => {
            // Return HX-Redirect header so HTMX follows the redirect
            (
                StatusCode::OK,
                [("HX-Redirect", format!("/seeds/{}", seed_id))],
                "",
            )
                .into_response()
        }
        Err(AppError::DuplicateSeed { existing_id }) => {
            let fragment = html! {
                div.duplicate-message {
                    p {
                        "This seed is already in your collection. "
                        a href=(format!("/seeds/{}", existing_id)) { "View it here" }
                    }
                }
            };
            (StatusCode::CONFLICT, fragment).into_response()
        }
        Err(e) => {
            let msg = match &e {
                AppError::ScraperError(s) => s.clone(),
                _ => "Could not extract seed data \u{2014} please try again".to_string(),
            };
            let fragment = html! {
                div.error-message {
                    p { (msg) }
                }
            };
            (StatusCode::INTERNAL_SERVER_ERROR, fragment).into_response()
        }
    }
}
