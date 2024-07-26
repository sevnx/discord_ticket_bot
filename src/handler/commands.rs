use std::time::Duration;

use poise::command;

use crate::handler::{Context, Error};

/// This command (`setup`) is used to set up a discord server.
#[command(slash_command, prefix_command)]
pub async fn setup(ctx: Context<'_>) -> Result<(), Error> {
    let mut pool = ctx.data().pool.acquire().await?;

    // Get the guild ID
    let guild_id = match ctx.guild_id() {
        Some(guild) => guild.get(),
        None => {
            return Err("Failed to get guild ID".into());
        }
    };

    // Check if the server is already set up
    let row = sqlx::query!(
        "SELECT id, setup_complete FROM servers WHERE id = $1",
        guild_id as i64
    )
    .fetch_optional(&mut *pool)
    .await?;

    match row {
        Some(row) => {
            if row.setup_complete {
                ctx.reply("Server already set up").await?;
                return Ok(());
            }
        }
        None => {
            sqlx::query!(
                "INSERT INTO servers (id, setup_complete) VALUES ($1, false)",
                guild_id as i64
            )
            .execute(&mut *pool)
            .await?;
        }
    }

    // Setting up the server

    let reply = ctx
        .author()
        .await_reply(&ctx)
        .timeout(Duration::from_secs(1))
        .await;

    match reply {
        Some(ref reply) => {
            ctx.say(format!("Reply: {}", reply.content)).await?;
        }
        None => {
            ctx.say("Timeout reached").await?;
        }
    }

    info!("{:?}", reply);

    Ok(())
}
