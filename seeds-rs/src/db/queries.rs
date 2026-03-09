use sqlx::SqlitePool;

use super::models::{Seed, SeasonPlan, SeedImage, SeedPurchase};

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

// --- Seed Purchase CRUD ---

pub async fn list_purchases_for_seed(
    pool: &SqlitePool,
    seed_id: i64,
) -> Result<Vec<SeedPurchase>, sqlx::Error> {
    sqlx::query_as::<_, SeedPurchase>(
        "SELECT * FROM seed_purchases WHERE seed_id = ? ORDER BY purchase_year DESC",
    )
    .bind(seed_id)
    .fetch_all(pool)
    .await
}

pub async fn get_purchase(
    pool: &SqlitePool,
    id: i64,
) -> Result<Option<SeedPurchase>, sqlx::Error> {
    sqlx::query_as::<_, SeedPurchase>("SELECT * FROM seed_purchases WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await
}

pub async fn insert_purchase(
    pool: &SqlitePool,
    seed_id: i64,
    purchase_year: i64,
    notes: Option<&str>,
) -> Result<i64, sqlx::Error> {
    let result = sqlx::query(
        "INSERT INTO seed_purchases (seed_id, purchase_year, notes) VALUES (?, ?, ?)",
    )
    .bind(seed_id)
    .bind(purchase_year)
    .bind(notes)
    .execute(pool)
    .await?;

    Ok(result.last_insert_rowid())
}

pub async fn update_purchase(
    pool: &SqlitePool,
    id: i64,
    purchase_year: i64,
    notes: Option<&str>,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query(
        "UPDATE seed_purchases SET purchase_year = ?, notes = ? WHERE id = ?",
    )
    .bind(purchase_year)
    .bind(notes)
    .bind(id)
    .execute(pool)
    .await?;

    Ok(result.rows_affected() > 0)
}

pub async fn delete_purchase(pool: &SqlitePool, id: i64) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("DELETE FROM seed_purchases WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;

    Ok(result.rows_affected() > 0)
}

/// Get the newest purchase year for each seed (for seed list display).
/// Returns (seed_id, newest_purchase_year) pairs.
pub async fn newest_purchase_per_seed(
    pool: &SqlitePool,
) -> Result<Vec<(i64, i64)>, sqlx::Error> {
    let rows = sqlx::query_as::<_, (i64, i64)>(
        "SELECT seed_id, MAX(purchase_year) FROM seed_purchases GROUP BY seed_id",
    )
    .fetch_all(pool)
    .await?;

    Ok(rows)
}

/// Count purchases per seed
pub async fn purchase_count_per_seed(
    pool: &SqlitePool,
) -> Result<Vec<(i64, i64)>, sqlx::Error> {
    let rows = sqlx::query_as::<_, (i64, i64)>(
        "SELECT seed_id, COUNT(*) FROM seed_purchases GROUP BY seed_id",
    )
    .fetch_all(pool)
    .await?;

    Ok(rows)
}

// --- Season Plan CRUD ---

/// List all season plans for a given year.
pub async fn list_season_plans(pool: &SqlitePool, year: i64) -> Result<Vec<SeasonPlan>, sqlx::Error> {
    sqlx::query_as::<_, SeasonPlan>("SELECT * FROM season_plans WHERE year = ? ORDER BY created_at DESC")
        .bind(year)
        .fetch_all(pool)
        .await
}

/// Check if a seed is in a given year's plan.
pub async fn is_seed_in_plan(pool: &SqlitePool, seed_id: i64, year: i64) -> Result<bool, sqlx::Error> {
    let row = sqlx::query_as::<_, (i64,)>(
        "SELECT COUNT(*) FROM season_plans WHERE seed_id = ? AND year = ?"
    )
    .bind(seed_id)
    .bind(year)
    .fetch_one(pool)
    .await?;

    Ok(row.0 > 0)
}

/// Toggle a seed in/out of a year's plan. Returns true if the seed is now in the plan.
pub async fn toggle_season_plan(pool: &SqlitePool, seed_id: i64, year: i64) -> Result<bool, sqlx::Error> {
    let exists = is_seed_in_plan(pool, seed_id, year).await?;

    if exists {
        sqlx::query("DELETE FROM season_plans WHERE seed_id = ? AND year = ?")
            .bind(seed_id)
            .bind(year)
            .execute(pool)
            .await?;
        Ok(false)
    } else {
        sqlx::query("INSERT INTO season_plans (seed_id, year) VALUES (?, ?)")
            .bind(seed_id)
            .bind(year)
            .execute(pool)
            .await?;
        Ok(true)
    }
}

/// List all seeds that are in a given year's plan (JOIN seeds with season_plans).
pub async fn list_planned_seeds(pool: &SqlitePool, year: i64) -> Result<Vec<Seed>, sqlx::Error> {
    sqlx::query_as::<_, Seed>(
        "SELECT s.* FROM seeds s INNER JOIN season_plans sp ON s.id = sp.seed_id WHERE sp.year = ? ORDER BY s.title"
    )
    .bind(year)
    .fetch_all(pool)
    .await
}

/// Get all planned seed IDs for a given year (efficient for set lookup).
pub async fn planned_seed_ids(pool: &SqlitePool, year: i64) -> Result<Vec<i64>, sqlx::Error> {
    let rows = sqlx::query_as::<_, (i64,)>(
        "SELECT seed_id FROM season_plans WHERE year = ?"
    )
    .bind(year)
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(|(id,)| id).collect())
}

pub async fn delete_seed(pool: &SqlitePool, id: i64) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("DELETE FROM seeds WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;

    Ok(result.rows_affected() > 0)
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
