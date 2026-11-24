-- Smart Trash: collection requests (the core table of the project).
-- Deliberately simple types — plain numbers and text — so the schema is
-- easy to read in a final year project. PostGIS can be added later.

-- Gives every pickup an id like PICKUP-00001.
CREATE SEQUENCE IF NOT EXISTS pickup_id_seq START 1;

CREATE TABLE IF NOT EXISTS pickups (
    pickup_id        TEXT PRIMARY KEY
                     DEFAULT ('PICKUP-' || LPAD(NEXTVAL('pickup_id_seq')::TEXT, 5, '0')),
    customer_id      TEXT        NOT NULL,               -- who asked for it
    waste_type       TEXT        NOT NULL,               -- ORGANIC / RECYCLABLE / ...
    latitude         DOUBLE PRECISION NOT NULL,          -- where
    longitude        DOUBLE PRECISION NOT NULL,
    estimated_volume DOUBLE PRECISION NOT NULL,          -- m³, > 0 (checked in Rust)
    collected_volume DOUBLE PRECISION NOT NULL DEFAULT 0, -- filled in on completion
    evidence_url     TEXT        NOT NULL DEFAULT '',    -- photo of the collection
    status           TEXT        NOT NULL DEFAULT 'PENDING',
    vehicle_id       TEXT        NOT NULL DEFAULT '',    -- set when assigned
    driver_id        TEXT        NOT NULL DEFAULT '',
    created_at_unix  BIGINT      NOT NULL DEFAULT 0,     -- seconds since 1970
    updated_at_unix  BIGINT      NOT NULL DEFAULT 0
);

-- Fast "show me all pending pickups" queries (dispatcher dashboard).
CREATE INDEX IF NOT EXISTS pickups_status_idx ON pickups (status);
