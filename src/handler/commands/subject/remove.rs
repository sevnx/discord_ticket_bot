use crate::handler::{commands::check_server_setup, Context, Error};
use poise::command;

/// Remove a subject from the list of subjects that can be used to better categorize tickets
#[command(
    slash_command,
    prefix_command,
    required_permissions = "MANAGE_CHANNELS",
    rename = "subjectremove",
    check = "check_server_setup",
    guild_only
)]
pub async fn remove(
    ctx: Context<'_>,
    #[description = "The subject to remove"]
    #[rest]
    name: String,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().ok_or("❌ - Guild ID not found")?;
    let mut pool = ctx.data().pool.acquire().await?;

    let subject = sqlx::query!(
        "SELECT name FROM subjects WHERE server_id = $1 AND name = $2",
        guild_id.get() as i64,
        name
    )
    .fetch_optional(&mut *pool)
    .await?;

    if subject.is_none() {
        ctx.reply("❌ - Subject not found").await?;
        return Ok(());
    }

    sqlx::query!(
        "DELETE FROM subjects WHERE server_id = $1 AND name = $2",
        guild_id.get() as i64,
        name
    )
    .execute(&mut *pool)
    .await?;

    ctx.reply("✅").await?;

    Ok(())
}
