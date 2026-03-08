use sqlx::SqlitePool;

use super::models::{Seed, SeedImage};

pub async fn list_seeds(pool: &SqlitePool) -> Result<Vec<Seed>, sqlx::Error> {
    sqlx::query_as::<_, Seed>("SELECT * FROM seeds ORDER BY created_at DESC")
        .fetch_all(pool)
        .await
}

pub async fn get_seed(pool: &SqlitePool, id: i64) -> Result<Option<Seed>, sqlx::Error> {
    sqlx::query_as::<_, Seed>("SELECT * FROM seeds WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await
}

pub async fn get_seed_images(
    pool: &SqlitePool,
    seed_id: i64,
) -> Result<Vec<SeedImage>, sqlx::Error> {
    sqlx::query_as::<_, SeedImage>(
        "SELECT * FROM seed_images WHERE seed_id = ? ORDER BY position",
    )
    .bind(seed_id)
    .fetch_all(pool)
    .await
}

pub async fn find_seed_by_handle(
    pool: &SqlitePool,
    handle: &str,
) -> Result<Option<Seed>, sqlx::Error> {
    sqlx::query_as::<_, Seed>("SELECT * FROM seeds WHERE product_handle = ?")
        .bind(handle)
        .fetch_optional(pool)
        .await
}

pub struct NewSeed {
    pub product_handle: String,
    pub source_url: String,
    pub title: String,
    pub description: Option<String>,
    pub category: Option<String>,
    pub subcategory: Option<String>,
    pub light_requirement: Option<String>,
    pub frost_tolerance: Option<String>,
    pub is_organic: bool,
    pub is_heirloom: bool,
    pub days_to_maturity: Option<String>,
    pub sow_depth: Option<String>,
    pub plant_spacing: Option<String>,
    pub germination_info: Option<String>,
    pub planting_instructions: Option<String>,
    pub growing_instructions: Option<String>,
    pub harvest_instructions: Option<String>,
    pub raw_html: Option<String>,
    pub shopify_product_id: Option<i64>,
    pub tags_raw: Option<String>,
}

pub async fn insert_seed(pool: &SqlitePool, seed: &NewSeed) -> Result<i64, sqlx::Error> {
    let result = sqlx::query(
        "INSERT INTO seeds (
            product_handle, source_url, title, description, category, subcategory,
            light_requirement, frost_tolerance, is_organic, is_heirloom,
            days_to_maturity, sow_depth, plant_spacing, germination_info,
            planting_instructions, growing_instructions, harvest_instructions,
            raw_html, shopify_product_id, tags_raw
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&seed.product_handle)
    .bind(&seed.source_url)
    .bind(&seed.title)
    .bind(&seed.description)
    .bind(&seed.category)
    .bind(&seed.subcategory)
    .bind(&seed.light_requirement)
    .bind(&seed.frost_tolerance)
    .bind(seed.is_organic)
    .bind(seed.is_heirloom)
    .bind(&seed.days_to_maturity)
    .bind(&seed.sow_depth)
    .bind(&seed.plant_spacing)
    .bind(&seed.germination_info)
    .bind(&seed.planting_instructions)
    .bind(&seed.growing_instructions)
    .bind(&seed.harvest_instructions)
    .bind(&seed.raw_html)
    .bind(seed.shopify_product_id)
    .bind(&seed.tags_raw)
    .execute(pool)
    .await?;

    Ok(result.last_insert_rowid())
}

pub struct NewSeedImage {
    pub seed_id: i64,
    pub shopify_image_id: Option<i64>,
    pub position: i64,
    pub original_url: String,
    pub local_filename: String,
    pub width: Option<i64>,
    pub height: Option<i64>,
}

pub async fn insert_image(pool: &SqlitePool, image: &NewSeedImage) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO seed_images (
            seed_id, shopify_image_id, position, original_url,
            local_filename, width, height
        ) VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(image.seed_id)
    .bind(image.shopify_image_id)
    .bind(image.position)
    .bind(&image.original_url)
    .bind(&image.local_filename)
    .bind(image.width)
    .bind(image.height)
    .execute(pool)
    .await?;

    Ok(())
}
