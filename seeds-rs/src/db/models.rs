use sqlx::SqlitePool;
use std::path::PathBuf;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Seed {
    pub id: i64,
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
    // purchase_year and notes columns still exist in DB but are deprecated;
    // new data goes to seed_purchases table
    pub purchase_year: Option<i64>,
    pub notes: Option<String>,
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct SeedPurchase {
    pub id: i64,
    pub seed_id: i64,
    pub purchase_year: i64,
    pub notes: Option<String>,
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct SeedImage {
    pub id: i64,
    pub seed_id: i64,
    pub shopify_image_id: Option<i64>,
    pub position: i64,
    pub original_url: String,
    pub local_filename: String,
    pub width: Option<i64>,
    pub height: Option<i64>,
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct SeasonPlan {
    pub id: i64,
    pub seed_id: i64,
    pub year: i64,
    pub notes: Option<String>,
    pub created_at: Option<String>,
}

#[derive(Debug, Clone)]
pub struct AppState {
    pub db: SqlitePool,
    pub data_dir: PathBuf,
}
