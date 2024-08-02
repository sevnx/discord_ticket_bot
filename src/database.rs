use std::env;

use poise::serenity_prelude::GuildId;
use sqlx::{
    postgres::{PgConnectOptions, PgPoolOptions},
    PgConnection,
};

use crate::handler::Error;

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

pub async fn is_server_setup(
    pool: &mut PgConnection,
    guild_id: GuildId,
) -> Result<Option<bool>, Error> {
    let row = sqlx::query!(
        "SELECT setup_complete FROM servers WHERE id = $1",
        guild_id.get() as i64
    )
    .fetch_optional(&mut *pool)
    .await?;
    Ok(row.map(|row| row.setup_complete))
}

/// Represents a subject
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct Subject {
    pub id: Option<u64>,
    pub name: String,
}

pub async fn get_subjects(
    pool: &mut PgConnection,
    guild_id: GuildId,
) -> Result<Vec<Subject>, Error> {
    info!("Getting subjects for guild {}", guild_id);
    let rows = sqlx::query!(
        "SELECT id, name FROM subjects WHERE server_id = $1",
        guild_id.get() as i64
    )
    .fetch_all(&mut *pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|row| Subject {
            id: Some(row.id as u64),
            name: row.name,
        })
        .collect())
}
