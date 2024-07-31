//! Adds a single subject to the list of subjects that can be used to better categorize tickets

use crate::{
    handler::{commands::check_server_setup, Context, Error},
    helper::parser::parse_discord_channel_id_url,
};
use poise::command;
use std::time::Duration;

/// Adds a single subject to the list of subjects that can be used to better categorize tickets
#[command(
    slash_command,
    required_permissions = "MANAGE_CHANNELS",
    rename = "subjectadd",
    check = "check_server_setup",
    guild_only
)]
pub async fn add_slash(
    ctx: Context<'_>,
    #[description = "The subject to add"] name: String,
    #[description = "The channel the subject is linked to to"] channel_id: String,
) -> Result<(), Error> {
    add_subject(ctx, name, channel_id).await
}

/// Adds a single subject to the list of subjects that can be used to better categorize tickets
#[command(
    prefix_command,
    required_permissions = "MANAGE_CHANNELS",
    check = "check_server_setup",
    aliases("subjectadd"),
    guild_only
)]
pub async fn add_prefix(ctx: Context<'_>, #[rest] name: Option<String>) -> Result<(), Error> {
    let Some(name) = name else {
        ctx.reply("Usage : `$subjectadd <subject>`").await?;
        return Ok(());
    };

    ctx.reply("Please provide the link to the channel you want to link the subject to")
        .await?;

    let Some(channel_id) = ctx
        .author()
        .await_reply(ctx)
        .timeout(Duration::from_secs(60))
        .await
    else {
        ctx.reply("❌ - No channel ID provided").await?;
        return Ok(());
    };

    add_subject(ctx, name, channel_id.content).await
}

async fn add_subject(ctx: Context<'_>, name: String, channel_id: String) -> Result<(), Error> {
    const MAX_SUBJECT_LENGTH: usize = 100;
    const MIN_SUBJECT_LENGTH: usize = 1;

    if name.len() > MAX_SUBJECT_LENGTH {
        ctx.reply("❌ - The subject is too long").await?;
        return Ok(());
    }
    if name.len() < MIN_SUBJECT_LENGTH {
        ctx.reply("❌ - The subject is too short").await?;
        return Ok(());
    }

    let Some(channel_id) = parse_discord_channel_id_url(&channel_id) else {
        ctx.reply("❌ - Invalid channel ID").await?;
        return Ok(());
    };

    let mut pool = ctx.data().pool.acquire().await?;
    let guild_id = ctx.guild_id().ok_or("Failed to get guild ID")?.get();

    if subject_exists(&mut pool, &name, guild_id).await? {
        ctx.reply("❌ - Subject already exists").await?;
        return Ok(());
    }

    sqlx::query!(
        "INSERT INTO subjects (name, server_id, channel_id) VALUES ($1, $2, $3)",
        name,
        guild_id as i64,
        channel_id as i64
    )
    .execute(&mut *pool)
    .await?;

    ctx.reply("✅").await?;

    Ok(())
}

async fn subject_exists(
    pool: &mut sqlx::PgConnection,
    name: &str,
    guild_id: u64,
) -> Result<bool, Error> {
    let row = sqlx::query!(
        "SELECT name FROM subjects WHERE name = $1 AND server_id = $2",
        name,
        guild_id as i64
    )
    .fetch_optional(pool)
    .await?;
    Ok(row.is_some())
}
