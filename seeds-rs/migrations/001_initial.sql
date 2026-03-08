PRAGMA journal_mode=WAL;

CREATE TABLE IF NOT EXISTS seeds (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    product_handle TEXT NOT NULL UNIQUE,
    source_url TEXT NOT NULL,
    title TEXT NOT NULL,
    description TEXT,
    category TEXT,
    subcategory TEXT,
    light_requirement TEXT,
    frost_tolerance TEXT,
    is_organic BOOLEAN DEFAULT FALSE,
    is_heirloom BOOLEAN DEFAULT FALSE,
    days_to_maturity TEXT,
    sow_depth TEXT,
    plant_spacing TEXT,
    germination_info TEXT,
    planting_instructions TEXT,
    growing_instructions TEXT,
    harvest_instructions TEXT,
    raw_html TEXT,
    shopify_product_id INTEGER,
    tags_raw TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS seed_images (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    seed_id INTEGER NOT NULL REFERENCES seeds(id) ON DELETE CASCADE,
    shopify_image_id INTEGER,
    position INTEGER NOT NULL,
    original_url TEXT NOT NULL,
    local_filename TEXT NOT NULL,
    width INTEGER,
    height INTEGER,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);
