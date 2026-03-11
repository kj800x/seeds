-- Add columns for all structured fields from the Botanical Interests
-- product page <p><b>Label:</b> Value</p> pattern.
ALTER TABLE seeds ADD COLUMN plant_type TEXT;
ALTER TABLE seeds ADD COLUMN botanical_name TEXT;
ALTER TABLE seeds ADD COLUMN family TEXT;
ALTER TABLE seeds ADD COLUMN native_region TEXT;
ALTER TABLE seeds ADD COLUMN hardiness TEXT;
ALTER TABLE seeds ADD COLUMN exposure TEXT;
ALTER TABLE seeds ADD COLUMN bloom_period TEXT;
ALTER TABLE seeds ADD COLUMN plant_dimensions TEXT;
ALTER TABLE seeds ADD COLUMN variety_info TEXT;
ALTER TABLE seeds ADD COLUMN attributes TEXT;
ALTER TABLE seeds ADD COLUMN when_to_sow_outside TEXT;
ALTER TABLE seeds ADD COLUMN when_to_start_inside TEXT;
ALTER TABLE seeds ADD COLUMN days_to_emerge TEXT;
ALTER TABLE seeds ADD COLUMN row_spacing TEXT;
ALTER TABLE seeds ADD COLUMN thinning TEXT;
ALTER TABLE seeds ADD COLUMN special_care TEXT;
