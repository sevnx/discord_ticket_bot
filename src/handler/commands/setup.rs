//! Setup command used to set up the bot in a server

use std::time::Duration;

use poise::{
    command,
    serenity_prelude::{
        model::channel, ChannelId, ChannelType, CreateChannel, CreateMessage, GuildChannel,
        ReactionType,
    },
};
use sqlx::PgConnection;

use crate::{
    handler::{commands::SimpleMessage, Context, Error},
    helper::parser::parse_discord_channel_id_url,
};

/// Constants for messages
mod messages {
    pub const MSG_SETUP_SUCCESS: &str = "Server set up successfully";
    pub const MSG_SETUP_ALREADY: &str = "Server already set up";
    pub const MSG_SETUP_TIMEOUT: &str = "Server setup timed out, please try again";
    pub const MSG_GUILD_FAIL: &str = "Failed to get guild ID";
    pub const MSG_SETUP_CHANNEL_INVALID: &str = "Invalid channel ID";
    pub const MSG_SETUP_CHANNEL_NOT_FOUND: &str = "Channel does not exist";
    pub const MSG_SETUP_CHANNEL_NOT_TEXT: &str = "Channel must be a text channel";
    pub const MSG_SETUP_CHANNEL_ID: &str =
        "Please provide the channel ID to be used for listening to requests of opening a ticket";
}

/// Setup the bot in your server
#[command(
    slash_command,
    prefix_command,
    required_permissions = "ADMINISTRATOR",
    guild_only
)]
pub async fn setup(ctx: Context<'_>) -> Result<(), Error> {
    let mut pool = ctx.data().pool.acquire().await?;

    let guild_id = ctx.guild_id().ok_or(messages::MSG_GUILD_FAIL)?.get();

    match is_server_setup(&mut pool, guild_id).await? {
        Some(true) => {
            ctx.send_simple_message(messages::MSG_SETUP_ALREADY).await?;
            return Ok(());
        }
        None => {
            sqlx::query!("INSERT INTO servers (id) VALUES ($1)", guild_id as i64)
                .execute(&mut *pool)
                .await?;
        }
        _ => {}
    }

    let channel_id = ask_for_ticket_channel_id(&ctx).await?;
    setup_request_channel(&ctx, &mut pool, guild_id, channel_id).await?;
    create_ticket_channel_categories(&ctx, &mut pool, guild_id).await?;
    setup_reaction_message(&ctx, &mut pool, guild_id, channel_id).await?;

    ctx.send_simple_message(messages::MSG_SETUP_SUCCESS).await?;

    sqlx::query!(
        "UPDATE servers SET setup_complete = true WHERE id = $1",
        guild_id as i64
    )
    .execute(&mut *pool)
    .await?;

    Ok(())
}

async fn ask_for_ticket_channel_id(ctx: &Context<'_>) -> Result<u64, Error> {
    ctx.send_simple_message(messages::MSG_SETUP_CHANNEL_ID)
        .await?;

    let Some(reply) = ctx
        .author()
        .await_reply(ctx)
        .timeout(Duration::from_secs(60))
        .await
    else {
        ctx.send_simple_message(messages::MSG_SETUP_TIMEOUT).await?;
        return Err(messages::MSG_SETUP_TIMEOUT.into());
    };

    let content = reply.content.trim();

    let channel_id: u64 = if let Some(channel_id) = parse_discord_channel_id_url(content) {
        channel_id
    } else {
        ctx.send_simple_message(messages::MSG_SETUP_CHANNEL_INVALID)
            .await?;
        return Err(messages::MSG_SETUP_CHANNEL_INVALID.into());
    };

    Ok(channel_id)
}

async fn is_server_setup(pool: &mut PgConnection, guild_id: u64) -> Result<Option<bool>, Error> {
    let row = sqlx::query!(
        "SELECT setup_complete FROM servers WHERE id = $1",
        guild_id as i64
    )
    .fetch_optional(&mut *pool)
    .await?;

    Ok(row.map(|row| row.setup_complete))
}

async fn setup_request_channel(
    ctx: &Context<'_>,
    pool: &mut PgConnection,
    guild_id: u64,
    channel_id: u64,
) -> Result<(), Error> {
    let Ok(channel) = ctx.http().get_channel(channel_id.into()).await else {
        ctx.send_simple_message(messages::MSG_SETUP_CHANNEL_NOT_FOUND)
            .await?;
        return Ok(());
    };

    if let channel::Channel::Guild(channel) = channel {
        if channel.kind == ChannelType::Voice {
            ctx.send_simple_message(messages::MSG_SETUP_CHANNEL_NOT_TEXT)
                .await?;
            return Ok(());
        }
    }

    sqlx::query!(
        "UPDATE servers SET ticket_channel_id = $1 WHERE id = $2",
        channel_id as i64,
        guild_id as i64
    )
    .execute(&mut *pool)
    .await?;

    Ok(())
}

/// Creates the ticket channel categories
///
/// Includes the unclaimed and claimed ticket categories
async fn create_ticket_channel_categories(
    ctx: &Context<'_>,
    pool: &mut PgConnection,
    guild_id: u64,
) -> Result<(), Error> {
    let unclaimed = create_server_category(ctx, guild_id, "Unclaimed Tickets").await?;
    let claimed = create_server_category(ctx, guild_id, "Claimed Tickets").await?;

    sqlx::query!(
        "UPDATE servers SET unclaimed_category_id = $1, claimed_category_id = $2 WHERE
        id = $3",
        unclaimed.id.get() as i64,
        claimed.id.get() as i64,
        guild_id as i64
    )
    .execute(&mut *pool)
    .await?;

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

async fn setup_reaction_message(
    ctx: &Context<'_>,
    pool: &mut PgConnection,
    guild_id: u64,
    channel_id: u64,
) -> Result<(), Error> {
    let message = CreateMessage::default().content("React with 🎫 to open a ticket");
    let sent_message = ChannelId::from(channel_id)
        .send_message(&ctx, message)
        .await?;

    sqlx::query!(
        "UPDATE servers SET ticket_message_id = $1 WHERE id = $2",
        sent_message.id.get() as i64,
        guild_id as i64
    )
    .execute(&mut *pool)
    .await?;

    sent_message
        .react(&ctx, ReactionType::Unicode("🎫".to_string()))
        .await?;

    Ok(())
}
