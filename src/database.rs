use std::env;

use sqlx::postgres::{PgConnectOptions, PgPoolOptions};

fn get_connection() -> Result<PgConnectOptions, String> {
    Ok(PgConnectOptions::new()
        .host(&env::var("DB_HOST").map_err(|error| error.to_string())?)
        .port(
            env::var("DB_PORT")
                .map_err(|error| error.to_string())?
                .parse()
                .map_err(|_| "Failed to parse port".to_string())?,
        )
        .username(&env::var("DB_USER").map_err(|error| error.to_string())?)
        .password(&env::var("DB_PASSWORD").map_err(|error| error.to_string())?)
        .database(&env::var("DB_NAME").map_err(|error| error.to_string())?))
}

pub async fn get_database_pool() -> Result<sqlx::PgPool, String> {
    let connection = get_connection()?;
    PgPoolOptions::new()
        .max_connections(5)
        .connect_with(connection)
        .await
        .map_err(|error| error.to_string())
}
