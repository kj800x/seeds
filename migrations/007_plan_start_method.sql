-- Track whether the user plans to start a seed indoors or outdoors.
-- Values: 'indoor', 'outdoor', or NULL (not yet chosen / use recommended).
ALTER TABLE season_plans ADD COLUMN start_method TEXT;
