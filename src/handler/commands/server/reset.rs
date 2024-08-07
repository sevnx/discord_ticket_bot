//! Setup command used to set up the bot in a server

use crate::{
    database::is_server_setup,
    handler::{
        commands::{check_server_setup, SimpleMessage},
        Context, Error,
    },
    helper::{embed::Custom, parser::parse_discord_channel_id_url},
    tickets::TICKET_EMOJI,
};
use poise::{
    command,
    serenity_prelude::{
        model::channel, ChannelId, ChannelType, CreateChannel, CreateEmbed, CreateMessage,
        GuildChannel, GuildId, MessageId, ReactionType, RoleId,
    },
};
use sqlx::PgConnection;
use std::time::Duration;

/// Reset the bot in a server
#[command(
    slash_command,
    prefix_command,
    required_permissions = "ADMINISTRATOR",
    check = "check_server_setup",
    guild_only
)]
pub async fn reset(ctx: Context<'_>) -> Result<(), Error> {
    let guild = ctx
        .guild_id()
        .ok_or("Failed to get guild ID")?
        .to_partial_guild(ctx.http())
        .await?;
    let mut pool = ctx.data().pool.acquire().await?;

    // Ask for confirmation

    // Delete the server from the database and return the server info

    Ok(())
}

/// Helper function for y/n confirmation
async fn get_confirmation(ctx: Context<'_>, question: &str) -> Result<bool, Error> {
    Ok(true)
}
