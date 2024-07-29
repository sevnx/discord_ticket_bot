//! This module regroups commands related to the subject of a ticket.

use crate::{
    database::is_server_setup,
    handler::{commands::SimpleMessage, Context, Error},
};
use poise::command;

mod messages {
    pub const MSG_GUILD_FAIL: &str = "Failed to get guild ID";
}

async fn check_server_setup(ctx: Context<'_>) -> Result<bool, Error> {
    let mut pool = ctx.data().pool.acquire().await?;
    let guild_id = ctx.guild_id().ok_or(messages::MSG_GUILD_FAIL)?.get();
    Ok(is_server_setup(&mut pool, guild_id).await? == Some(true))
}

/// Add a subject to the list of subjects that can be used to better categorize tickets
#[command(
    slash_command,
    prefix_command,
    required_permissions = "MANAGE_CHANNELS",
    rename = "subjectadd",
    check = "check_server_setup",
    guild_only
)]
pub async fn add(
    ctx: Context<'_>,
    #[description = "The list os subjects to be added separated by `$`"]
    #[rest]
    input: String,
) -> Result<(), Error> {
    const MAX_SUBJECT_LENGTH: usize = 100;
    const MIN_SUBJECT_LENGTH: usize = 1;
    let subjects = input.split('$').map(|s| s.trim()).collect::<Vec<&str>>();
    let mut pool = ctx.data().pool.acquire().await?;
    let guild_id = ctx.guild_id().ok_or(messages::MSG_GUILD_FAIL)?.get();
    let mut response = String::new();

    for subject in subjects {
        if subject.len() > MAX_SUBJECT_LENGTH || subject.len() < MIN_SUBJECT_LENGTH {
            response.push_str(&format!(
                "❌ - `{}` - must be between {} and {} characters long\n",
                subject, MIN_SUBJECT_LENGTH, MAX_SUBJECT_LENGTH
            ));
            continue;
        }

        let row = sqlx::query!(
            "SELECT name FROM subjects WHERE name = $1 AND server_id = $2",
            subject,
            guild_id as i64
        )
        .fetch_optional(&mut *pool)
        .await?;

        if row.is_some() {
            response.push_str(&format!("❌ - `{}` - already exists\n", subject));
            continue;
        }

        sqlx::query!(
            "INSERT INTO subjects (name, server_id) VALUES ($1, $2)",
            subject,
            guild_id as i64
        )
        .execute(&mut *pool)
        .await?;

        response.push_str(&format!("✅ - `{}`\n", subject));
    }

    ctx.reply(response).await?;

    Ok(())
}

/// List all the subjects that can be used to better categorize tickets
#[command(
    slash_command,
    prefix_command,
    required_permissions = "MANAGE_CHANNELS",
    rename = "subjectlist",
    check = "check_server_setup",
    guild_only
)]
pub async fn list(ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

/// Remove a subject from the list of subjects that can be used to better categorize tickets
#[command(
    slash_command,
    prefix_command,
    required_permissions = "MANAGE_CHANNELS",
    rename = "subjectremove",
    check = "check_server_setup",
    guild_only
)]
pub async fn remove(ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}
