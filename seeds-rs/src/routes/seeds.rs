use axum::extract::{Form, Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use maud::{html, Markup};
use serde::Deserialize;

use crate::db::models::AppState;
use crate::db::queries;
use crate::error::AppError;
use crate::scraper;
use crate::templates::seed_detail::{seed_detail_page, seed_purchases_section};

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
    let purchases = queries::list_purchases_for_seed(&state.db, id).await?;

    Ok(seed_detail_page(&seed, &images, &purchases))
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
            // If purchase_year was provided, create a purchase record
            if let Some(year) = input.purchase_year {
                let _ = queries::insert_purchase(&state.db, seed_id, year, None).await;
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

// --- Purchase CRUD handlers ---

#[derive(Deserialize)]
pub struct AddPurchaseInput {
    pub purchase_year: i64,
    pub notes: Option<String>,
}

/// POST /seeds/{id}/purchases - Add a new purchase record
pub async fn add_purchase_handler(
    State(state): State<AppState>,
    Path(seed_id): Path<i64>,
    Form(input): Form<AddPurchaseInput>,
) -> Result<Markup, AppError> {
    let seed = queries::get_seed(&state.db, seed_id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Seed {} not found", seed_id)))?;

    queries::insert_purchase(&state.db, seed_id, input.purchase_year, input.notes.as_deref())
        .await?;

    let purchases = queries::list_purchases_for_seed(&state.db, seed_id).await?;
    Ok(seed_purchases_section(&seed, &purchases))
}

#[derive(Deserialize)]
pub struct UpdatePurchaseInput {
    pub purchase_year: i64,
    pub notes: Option<String>,
}

/// PUT /seeds/{seed_id}/purchases/{purchase_id} - Update a purchase record
pub async fn update_purchase_handler(
    State(state): State<AppState>,
    Path((seed_id, purchase_id)): Path<(i64, i64)>,
    Form(input): Form<UpdatePurchaseInput>,
) -> Result<Markup, AppError> {
    let seed = queries::get_seed(&state.db, seed_id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Seed {} not found", seed_id)))?;

    queries::update_purchase(&state.db, purchase_id, input.purchase_year, input.notes.as_deref())
        .await?;

    let purchases = queries::list_purchases_for_seed(&state.db, seed_id).await?;
    Ok(seed_purchases_section(&seed, &purchases))
}

/// DELETE /seeds/{seed_id}/purchases/{purchase_id} - Delete a purchase record
pub async fn delete_purchase_handler(
    State(state): State<AppState>,
    Path((seed_id, purchase_id)): Path<(i64, i64)>,
) -> Result<Markup, AppError> {
    let seed = queries::get_seed(&state.db, seed_id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Seed {} not found", seed_id)))?;

    queries::delete_purchase(&state.db, purchase_id).await?;

    let purchases = queries::list_purchases_for_seed(&state.db, seed_id).await?;
    Ok(seed_purchases_section(&seed, &purchases))
}

/// GET /seeds/{seed_id}/purchases/{purchase_id}/edit - Return edit form for a purchase
pub async fn edit_purchase_form(
    State(state): State<AppState>,
    Path((seed_id, purchase_id)): Path<(i64, i64)>,
) -> Result<Markup, AppError> {
    let _seed = queries::get_seed(&state.db, seed_id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Seed {} not found", seed_id)))?;

    let purchase = queries::get_purchase(&state.db, purchase_id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Purchase {} not found", purchase_id)))?;

    Ok(html! {
        tr.purchase-edit-row {
            td colspan="4" {
                form hx-put=(format!("/seeds/{}/purchases/{}", seed_id, purchase_id))
                     hx-target="#seed-purchases" hx-swap="outerHTML" {
                    div.edit-form-fields.edit-form-inline {
                        div.form-field {
                            label for="purchase_year" { "Year" }
                            input type="number" name="purchase_year" value=(purchase.purchase_year)
                                  min="2000" max="2030" required;
                        }
                        div.form-field {
                            label for="notes" { "Notes" }
                            input type="text" name="notes"
                                  value=[purchase.notes.as_deref()]
                                  placeholder="Optional notes...";
                        }
                        div.edit-form-actions {
                            button type="submit" class="btn btn-save btn-sm" { "Save" }
                            button type="button" class="btn btn-cancel btn-sm"
                                   hx-get=(format!("/seeds/{}/purchases", seed_id))
                                   hx-target="#seed-purchases" hx-swap="outerHTML" { "Cancel" }
                        }
                    }
                }
            }
        }
    })
}

/// GET /seeds/{id}/purchases - Return the purchases section fragment
pub async fn purchases_fragment(
    State(state): State<AppState>,
    Path(seed_id): Path<i64>,
) -> Result<Markup, AppError> {
    let seed = queries::get_seed(&state.db, seed_id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Seed {} not found", seed_id)))?;

    let purchases = queries::list_purchases_for_seed(&state.db, seed_id).await?;
    Ok(seed_purchases_section(&seed, &purchases))
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
