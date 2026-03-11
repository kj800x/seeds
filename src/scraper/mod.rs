pub mod fetcher;
pub mod images;
pub mod parser;

use crate::db::models::AppState;
use crate::db::queries::{self, NewSeed, NewSeedImage};
use crate::error::AppError;

/// Result of a duplicate URL check -- carries the existing seed ID.
#[allow(dead_code)]
#[derive(Debug)]
pub struct DuplicateSeed {
    pub existing_id: i64,
}

/// Scrape a Botanical Interests product page and persist all data to the database.
///
/// This is the main orchestrator function that:
/// 1. Validates the URL and checks for duplicates
/// 2. Fetches JSON API + HTML page (dual-fetch strategy)
/// 3. Parses tags for structured data (category, light, etc.)
/// 4. Parses HTML for growing details (best-effort)
/// 5. Inserts the seed record including raw HTML (SCRP-05)
/// 6. Downloads product images to filesystem
/// 7. Inserts image records into the database
///
/// Returns the new seed's database ID on success.
pub async fn scrape_and_save(state: &AppState, url: &str) -> Result<i64, AppError> {
    let client = reqwest::Client::builder()
        .user_agent("SeedsApp/1.0 (personal garden planner)")
        .build()
        .map_err(|e| AppError::ScraperError(format!("Failed to build HTTP client: {}", e)))?;

    // Fetch product data (JSON + HTML) and extract the handle
    let (product, raw_html, handle) = fetcher::fetch_product(&client, url).await?;

    // Check for duplicate by product handle
    if let Some(existing) = queries::find_seed_by_handle(&state.db, &handle).await? {
        return Err(AppError::DuplicateSeed {
            existing_id: existing.id,
        });
    }

    // Parse tags for structured data
    let tags = parser::parse_tags(&product.tags);

    // Parse HTML for growing details (best-effort)
    let growing = parser::parse_growing_details(&raw_html);

    // Build the source URL in canonical form
    let source_url = format!(
        "https://www.botanicalinterests.com/products/{}",
        handle
    );

    // Insert the seed record
    let new_seed = NewSeed {
        product_handle: handle,
        source_url,
        title: product.title,
        description: product.body_html,
        category: tags.category,
        subcategory: tags.subcategory,
        light_requirement: tags.light_requirement,
        frost_tolerance: tags.frost_tolerance,
        is_organic: tags.is_organic,
        is_heirloom: tags.is_heirloom,
        days_to_maturity: growing.days_to_maturity,
        sow_depth: growing.sow_depth,
        plant_spacing: growing.plant_spacing,
        germination_info: growing.germination_info,
        planting_instructions: growing.planting_instructions,
        growing_instructions: growing.growing_instructions,
        harvest_instructions: growing.harvest_instructions,
        raw_html: Some(raw_html),
        shopify_product_id: Some(product.id),
        tags_raw: Some(product.tags),
        plant_type: growing.plant_type,
        botanical_name: growing.botanical_name,
        family: growing.family,
        native_region: growing.native_region,
        hardiness: growing.hardiness,
        exposure: growing.exposure,
        bloom_period: growing.bloom_period,
        plant_dimensions: growing.plant_dimensions,
        variety_info: growing.variety_info,
        attributes: growing.attributes,
        when_to_sow_outside: growing.when_to_sow_outside,
        when_to_start_inside: growing.when_to_start_inside,
        days_to_emerge: growing.days_to_emerge,
        row_spacing: growing.row_spacing,
        thinning: growing.thinning,
        special_care: growing.special_care,
    };

    let seed_id = queries::insert_seed(&state.db, &new_seed).await?;

    // Download images to filesystem
    let downloaded = images::download_images(
        &client,
        &product.images,
        seed_id,
        &state.data_dir,
    )
    .await;

    // Insert image records
    for img in &downloaded {
        let new_image = NewSeedImage {
            seed_id,
            shopify_image_id: Some(img.shopify_image_id),
            position: img.position as i64,
            original_url: img.original_url.clone(),
            local_filename: img.filename.clone(),
            width: img.width.map(|w| w as i64),
            height: img.height.map(|h| h as i64),
        };

        if let Err(e) = queries::insert_image(&state.db, &new_image).await {
            tracing::warn!("Failed to insert image record for seed {}: {}", seed_id, e);
            // Continue -- image metadata failure shouldn't fail the whole scrape
        }
    }

    tracing::info!(
        "Successfully scraped seed {} (id={}), {} images downloaded",
        new_seed.title,
        seed_id,
        downloaded.len()
    );

    Ok(seed_id)
}
