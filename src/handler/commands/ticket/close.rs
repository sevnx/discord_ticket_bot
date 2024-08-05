use crate::{
    handler::{commands::check_server_setup, Context, Error},
    tickets,
};
use poise::command;

/// Closes a ticket
#[command(
    slash_command,
    prefix_command,
    required_permissions = "MANAGE_CHANNELS",
    check = "check_server_setup",
    guild_only
)]
pub async fn close(ctx: Context<'_>) -> Result<(), Error> {
    if let Err(error) = tickets::close::close(&ctx).await {
        error!("Error closing ticket: {}", error);
    }

    Ok(())
}
