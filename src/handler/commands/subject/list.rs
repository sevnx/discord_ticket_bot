use crate::handler::{commands::check_server_setup, Context, Error};
use poise::command;

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
    let guild_id = ctx.guild_id().ok_or("❌ - Guild ID not found")?;
    let mut pool = ctx.data().pool.acquire().await?;

    let subjects = sqlx::query!(
        "SELECT name, channel_id FROM subjects WHERE server_id = $1",
        guild_id.get() as i64
    )
    .fetch_all(&mut *pool)
    .await?;

    let answer = subjects
        .iter()
        .map(|subject| format!("- {} - <#{}>", subject.name, subject.channel_id))
        .collect::<Vec<String>>()
        .join("\n");

    if answer.is_empty() {
        ctx.reply("❌ - No subjects found").await?;
        return Ok(());
    }

    ctx.reply(answer).await?;

    Ok(())
}
