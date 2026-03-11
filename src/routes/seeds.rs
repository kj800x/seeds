use axum::extract::{Form, Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use maud::{html, Markup};
use serde::Deserialize;

use chrono::Datelike;

use crate::db::models::AppState;
use crate::db::queries;
use crate::error::AppError;
use crate::scraper;
use crate::templates::home::plan_toggle_button;
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

    let current_year = chrono::Local::now().year() as i64;
    let in_plan = queries::is_seed_in_plan(&state.db, id, current_year).await?;
    let plan_start_method = if in_plan {
        Some(queries::get_plan_start_method(&state.db, id, current_year).await?.unwrap_or_default())
    } else {
        None
    };

    Ok(seed_detail_page(&seed, &images, &purchases, in_plan, plan_start_method.as_deref()))
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

/// POST /seeds/reparse - Re-parse all seeds from stored raw_html
pub async fn reparse_all(
    State(state): State<AppState>,
) -> Result<Response, AppError> {
    let count = queries::reparse_all_seeds(&state.db).await?;
    tracing::info!("Re-parsed {} seeds from stored HTML", count);

    Ok((
        StatusCode::OK,
        [("HX-Redirect", "/".to_string())],
        format!("Re-parsed {} seeds", count),
    )
        .into_response())
}

/// Query params for toggle_plan
#[derive(Deserialize)]
pub struct TogglePlanQuery {
    pub detail: Option<u8>,
}

/// Determine the default start method for a seed based on its timing data.
fn default_start_method(seed: &crate::db::models::Seed) -> Option<&'static str> {
    let timing = crate::schedule::parse_planting_timing_from_fields(
        seed.when_to_sow_outside.as_deref(),
        seed.when_to_start_inside.as_deref(),
    );
    let has_indoor = timing.start_indoors_weeks_before.is_some();
    let has_outdoor = timing.direct_sow_weeks_relative.is_some();
    match (has_indoor, has_outdoor) {
        (true, true) => Some(if timing.indoor_start_recommended { "indoor" } else { "outdoor" }),
        (true, false) => Some("indoor"),
        (false, true) => Some("outdoor"),
        (false, false) => None,
    }
}

/// POST /plan/toggle/{seed_id} - Toggle a seed in/out of this year's season plan
pub async fn toggle_plan(
    State(state): State<AppState>,
    Path(seed_id): Path<i64>,
    axum::extract::Query(query): axum::extract::Query<TogglePlanQuery>,
) -> Result<Markup, AppError> {
    let current_year = chrono::Local::now().year() as i64;
    let in_plan = queries::toggle_season_plan(&state.db, seed_id, current_year).await?;

    // When adding to plan, set the default start method based on recommendations
    if in_plan
        && let Some(seed) = queries::get_seed(&state.db, seed_id).await?
        && let Some(method) = default_start_method(&seed)
    {
        queries::update_plan_start_method(&state.db, seed_id, current_year, Some(method)).await?;
    }

    // On the detail page, re-render the full timeline section so method selector appears
    if query.detail == Some(1) {
        let seed = queries::get_seed(&state.db, seed_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Seed {} not found", seed_id)))?;

        let timing = crate::schedule::parse_planting_timing_from_fields(
            seed.when_to_sow_outside.as_deref(),
            seed.when_to_start_inside.as_deref(),
        );

        let current_year_i32 = current_year as i32;
        let indoor = crate::schedule::compute_indoor_timeline(&seed, &timing, current_year_i32);
        let outdoor = crate::schedule::compute_outdoor_timeline(&seed, &timing, current_year_i32);

        if indoor.is_some() && outdoor.is_some() {
            let plan_start_method = if in_plan {
                Some(queries::get_plan_start_method(&state.db, seed_id, current_year).await?.unwrap_or_default())
            } else {
                None
            };
            return Ok(crate::templates::schedule::seed_detail_dual_timeline(
                &seed, &timing, current_year_i32, in_plan, plan_start_method.as_deref(),
            ));
        } else {
            return Ok(crate::templates::schedule::seed_detail_timeline(
                &seed, &timing, current_year_i32, in_plan,
            ));
        }
    }

    Ok(plan_toggle_button(seed_id, in_plan))
}

#[derive(Deserialize)]
pub struct StartMethodInput {
    pub method: String,
}

/// POST /plan/{seed_id}/start-method - Set indoor/outdoor start method for a planned seed
pub async fn set_start_method(
    State(state): State<AppState>,
    Path(seed_id): Path<i64>,
    Form(input): Form<StartMethodInput>,
) -> Result<Markup, AppError> {
    let current_year = chrono::Local::now().year() as i64;

    let method = match input.method.as_str() {
        "indoor" | "outdoor" => Some(input.method.as_str()),
        _ => None,
    };

    queries::update_plan_start_method(&state.db, seed_id, current_year, method).await?;

    // Re-render the timeline section
    let seed = queries::get_seed(&state.db, seed_id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Seed {} not found", seed_id)))?;

    let current_year_i32 = current_year as i32;
    let timing = crate::schedule::parse_planting_timing_from_fields(
        seed.when_to_sow_outside.as_deref(),
        seed.when_to_start_inside.as_deref(),
    );

    Ok(crate::templates::schedule::seed_detail_dual_timeline(
        &seed, &timing, current_year_i32, true, method,
    ))
}
