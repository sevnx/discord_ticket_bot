use crate::{
    handler::{commands::check_server_setup, Context, Error},
    tickets,
};
use poise::command;

/// Claims a ticket
#[command(
    slash_command,
    prefix_command,
    check = "check_server_setup",
    guild_only
)]
pub async fn claim(ctx: Context<'_>) -> Result<(), Error> {
    match tickets::claim::claim(&ctx).await {
        Ok(()) => {
            ctx.reply("✅").await?;
        }
        Err(e) => {
            error!("Error claiming ticket: {}", e);
        }
    }

    Ok(())
}
