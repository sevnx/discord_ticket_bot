//! Setup command used to set up the bot in a server

use std::time::Duration;

use poise::{
    command,
    serenity_prelude::{model::channel, ChannelType, CreateChannel, GuildChannel},
};

use crate::{
    handler::{commands::SimpleMessage, Context, Error},
    helper::parser::parse_discord_channel_id_url,
};

// Constants (for messages)

/// Message to be sent when the server is set up successfully
const MSG_SETUP_SUCCESS: &str = "Server set up successfully";

/// Message to be sent when the server is already set up
const MSG_SETUP_ALREADY: &str = "Server already set up";

/// Message to be sent when the server setup times out
const MSG_SETUP_TIMEOUT: &str = "Server setup timed out, please try again";

/// Message to be sent when asking for the channel id to be used for listening to request of
/// opening a ticket
const MSG_SETUP_CHANNEL_ID: &str =
    "Please provide the channel ID to be used for listening to requests of opening a ticket";

/// Message to be sent when the user does not have the required permissions to run a command
const MSG_NO_PERMISSIONS: &str = "You do not have the required permissions to run this command";

/// Setup the bot in your server
#[command(slash_command, prefix_command)]
pub async fn setup(ctx: Context<'_>) -> Result<(), Error> {
    let mut pool = ctx.data().pool.acquire().await?;

    let author = ctx
        .author_member()
        .await
        .ok_or("Failed to get author member")?;

    if let Ok(permissions) = author.permissions(ctx.cache()) {
        if !permissions.administrator() {
            ctx.send_simple_message(MSG_NO_PERMISSIONS).await?;
            return Ok(());
        }
    }

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
                ctx.send_simple_message(MSG_SETUP_ALREADY).await?;
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

    // Ask for the channel ID to be used for listening to requests of opening a ticket
    ctx.send_simple_message(MSG_SETUP_CHANNEL_ID).await?;

    // Wait for the reply
    let Some(reply) = ctx
        .author()
        .await_reply(ctx)
        .timeout(Duration::from_secs(60))
        .await
    else {
        ctx.send_simple_message(MSG_SETUP_TIMEOUT).await?;
        return Ok(());
    };

    let content = reply.content.trim();

    let channel_id = match content.parse::<u64>() {
        Ok(channel_id) => channel_id,
        Err(_) => {
            if let Some(channel_id) = parse_discord_channel_id_url(content) {
                channel_id
            } else {
                ctx.send_simple_message("Invalid channel ID").await?;
                return Ok(());
            }
        }
    };

    // Check if the channel exists
    let Ok(channel) = ctx.http().get_channel(channel_id.into()).await else {
        ctx.send_simple_message("Channel does not exist").await?;
        return Ok(());
    };

    // Check if the channel is a text channel
    if let channel::Channel::Guild(channel) = channel {
        if channel.kind == ChannelType::Voice {
            ctx.send_simple_message("Channel must be a text channel")
                .await?;
            return Ok(());
        }
    }

    // Insert the channel ID into the database
    let query = sqlx::query!(
        "UPDATE servers SET ticket_channel_id = $1 WHERE id = $2",
        channel_id as i64,
        guild_id as i64
    )
    .execute(&mut *pool)
    .await?;

    // Check if the query was successful
    if query.rows_affected() == 0 {
        ctx.send_simple_message("Failed to set up server").await?;
        return Ok(());
    }

    let unclaimed = create_server_category(&ctx, guild_id, "Unclaimed Tickets").await?;
    let claimed = create_server_category(&ctx, guild_id, "Claimed Tickets").await?;

    // Add to the database
    let query = sqlx::query!(
        "UPDATE servers SET unclaimed_category_id = $1, claimed_category_id = $2 WHERE id = $3",
        unclaimed.id.get() as i64,
        claimed.id.get() as i64,
        guild_id as i64
    );

    query.execute(&mut *pool).await?;

    // TODO: Send the create channel message

    // TODO: Change the setup status to true (once the entire code is implemented)

    ctx.send_simple_message(MSG_SETUP_SUCCESS).await?;

    Ok(())
}

/// Creates a server category
async fn create_server_category(
    ctx: &Context<'_>,
    guild_id: u64,
    name: &str,
) -> Result<GuildChannel, Error> {
    let builder = CreateChannel::new("")
        .name(name)
        .kind(ChannelType::Category);

    let category = ctx
        .http()
        .create_channel(guild_id.into(), &builder, None)
        .await?;

    Ok(category)
}
