-- Add status column to season_plans: 'active' (default) or 'skipped'
ALTER TABLE season_plans ADD COLUMN status TEXT NOT NULL DEFAULT 'active';
