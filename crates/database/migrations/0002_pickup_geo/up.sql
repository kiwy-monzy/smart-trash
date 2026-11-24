-- Geospatial columns for pickups.
ALTER TABLE pickups ADD COLUMN IF NOT EXISTS h3_cell TEXT NOT NULL DEFAULT '';
ALTER TABLE pickups ADD COLUMN IF NOT EXISTS ward TEXT NOT NULL DEFAULT '';

CREATE INDEX IF NOT EXISTS pickups_h3_idx ON pickups (h3_cell);
CREATE INDEX IF NOT EXISTS pickups_customer_idx ON pickups (customer_id);
