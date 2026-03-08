-- Create seed_purchases table for tracking multiple purchase lots per seed
CREATE TABLE IF NOT EXISTS seed_purchases (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    seed_id INTEGER NOT NULL REFERENCES seeds(id) ON DELETE CASCADE,
    purchase_year INTEGER NOT NULL,
    notes TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Migrate existing purchase_year/notes data from seeds table
INSERT INTO seed_purchases (seed_id, purchase_year, notes, created_at)
SELECT id, purchase_year, notes, created_at
FROM seeds
WHERE purchase_year IS NOT NULL;
