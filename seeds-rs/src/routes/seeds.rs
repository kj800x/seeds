use axum::extract::{Form, Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use maud::{html, Markup};
use serde::Deserialize;

use crate::db::models::AppState;
use crate::db::queries;
use crate::error::AppError;
use crate::scraper;
use crate::templates::seed_detail::{seed_detail_page, seed_info_section};

#[derive(Deserialize)]
pub struct AddSeedInput {
    pub url: String,
    pub purchase_year: Option<i64>,
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
            // If purchase_year was provided, update the seed record
            if input.purchase_year.is_some() {
                let _ = queries::update_seed(&state.db, seed_id, input.purchase_year, None).await;
            }

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

#[derive(Deserialize)]
pub struct UpdateSeedInput {
    pub purchase_year: Option<i64>,
    pub notes: Option<String>,
}

/// GET /seeds/{id}/edit - Returns inline edit form fragment
pub async fn edit_seed_form(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Markup, AppError> {
    let seed = queries::get_seed(&state.db, id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Seed {} not found", id)))?;

    Ok(html! {
        form #seed-info hx-put=(format!("/seeds/{}", seed.id)) hx-target="this" hx-swap="outerHTML" {
            section.detail-section {
                h2 { "Edit Inventory Info" }
                div.edit-form-fields {
                    div.form-field {
                        label for="purchase_year" { "Purchase Year" }
                        input type="number" name="purchase_year" id="purchase_year"
                              value=[seed.purchase_year.map(|y| y.to_string())]
                              placeholder="e.g. 2025" min="2000" max="2030";
                    }
                    div.form-field {
                        label for="notes" { "Notes" }
                        textarea name="notes" id="notes" rows="3"
                                 placeholder="Add notes about this seed..." {
                            @if let Some(ref notes) = seed.notes {
                                (notes)
                            }
                        }
                    }
                }
                div.edit-form-actions {
                    button type="submit" class="btn btn-save" { "Save" }
                    button type="button" class="btn btn-cancel"
                           hx-get=(format!("/seeds/{}/info", seed.id))
                           hx-target="#seed-info" hx-swap="outerHTML" { "Cancel" }
                }
            }
        }
    })
}

/// GET /seeds/{id}/info - Returns display-mode seed info fragment
pub async fn seed_info_fragment(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Markup, AppError> {
    let seed = queries::get_seed(&state.db, id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Seed {} not found", id)))?;

    Ok(seed_info_section(&seed))
}

/// PUT /seeds/{id} - Update seed and return display-mode fragment
pub async fn update_seed_handler(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Form(input): Form<UpdateSeedInput>,
) -> Result<Markup, AppError> {
    queries::update_seed(&state.db, id, input.purchase_year, input.notes.as_deref()).await?;

    let seed = queries::get_seed(&state.db, id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Seed {} not found", id)))?;

    Ok(seed_info_section(&seed))
}

/// DELETE /seeds/{id} - Delete seed and redirect to list
pub async fn delete_seed_handler(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Response, AppError> {
    queries::delete_seed(&state.db, id).await?;

    // Clean up image files (ignore errors -- orphaned files are harmless)
    let image_dir = state.data_dir.join("images").join(id.to_string());
    let _ = std::fs::remove_dir_all(&image_dir);

    Ok((
        StatusCode::OK,
        [("HX-Redirect", "/".to_string())],
        "",
    )
        .into_response())
}
