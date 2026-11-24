//! PostgreSQL access for the project: connection, migrations and Diesel models.

pub mod models;
pub mod schema;

use diesel::pg::PgConnection;
use diesel::Connection;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

/// Migrations compiled into the binary.
pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

/// Open a connection using `DATABASE_URL` from `.env`.
pub fn establish_connection() -> PgConnection {
    let url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL is not set — check the .env file in the project root");

    PgConnection::establish(&url).unwrap_or_else(|err| {
        panic!(
            "could not connect to PostgreSQL at {url}: {err}\n\
             Is the postgresql-x64 service running? (.\\build.ps1 sets up libpq)"
        )
    })
}

/// Apply pending migrations; already applied ones are skipped.
pub fn run_migrations(connection: &mut PgConnection) {
    connection
        .run_pending_migrations(MIGRATIONS)
        .unwrap_or_else(|err| panic!("database migrations failed: {err}"));

    tracing::info!("database migrations are up to date");
}

#[cfg(test)]
mod tests {
    #[test]
    fn migration_files_exist() {
        // cargo test runs with this crate as the working directory.
        assert!(std::path::Path::new("migrations/0001_init/up.sql").exists());
        assert!(std::path::Path::new("migrations/0001_init/down.sql").exists());
    }

    #[test]
    fn schema_matches_the_migration() {
        // Every column the migration creates must exist in schema.rs.
        let up = std::fs::read_to_string("migrations/0001_init/up.sql").unwrap();
        for column in [
            "pickup_id",
            "customer_id",
            "waste_type",
            "latitude",
            "longitude",
            "estimated_volume",
            "collected_volume",
            "evidence_url",
            "status",
            "vehicle_id",
            "driver_id",
            "created_at_unix",
            "updated_at_unix",
        ] {
            assert!(up.contains(column), "migration is missing {column}");
        }
    }
}
