-- Track actual planting events (when user actually sowed/transplanted)
-- per seed per season plan year.
CREATE TABLE season_plan_events (
    id INTEGER PRIMARY KEY,
    seed_id INTEGER NOT NULL REFERENCES seeds(id) ON DELETE CASCADE,
    year INTEGER NOT NULL,
    event_type TEXT NOT NULL, -- 'sow_indoor', 'sow_outdoor', 'transplant'
    event_date TEXT NOT NULL, -- ISO date string 'YYYY-MM-DD'
    notes TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_season_plan_events_seed_year ON season_plan_events(seed_id, year);
