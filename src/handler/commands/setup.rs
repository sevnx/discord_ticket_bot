//! Setup command used to set up the bot in a server

use std::time::Duration;

use poise::{
    command,
    serenity_prelude::{
        model::channel, ChannelId, ChannelType, CreateChannel, CreateEmbed, CreateMessage,
        GuildChannel, GuildId, ReactionType,
    },
};
use roles::get_new_or_existing_role;
use sqlx::PgConnection;

use crate::{
    database::is_server_setup,
    handler::{commands::SimpleMessage, Context, Error},
    helper::{embed::Custom, parser::parse_discord_channel_id_url},
    tickets::TICKET_EMOJI,
};

mod roles;

/// Constants for messages
// TODO: Do something about this, to unify handling across the bot
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

    let guild_id = ctx.guild_id().ok_or(messages::MSG_GUILD_FAIL)?;

    match is_server_setup(&mut pool, guild_id).await? {
        Some(true) => {
            ctx.send_simple_message(messages::MSG_SETUP_ALREADY).await?;
            return Ok(());
        }
        None => {
            sqlx::query!(
                "INSERT INTO servers (id) VALUES ($1)",
                guild_id.get() as i64
            )
            .execute(&mut *pool)
            .await?;
        }
        _ => {}
    }

    let ticket_channel = ask_for_ticket_channel_id(&ctx).await?;
    setup_log_channel(&ctx, &mut pool).await?;
    setup_request_channel(&ctx, &mut pool, guild_id, ticket_channel).await?;
    create_ticket_channel_categories(&ctx, &mut pool, guild_id).await?;
    setup_helper_role(&ctx, &mut pool, guild_id).await?;
    setup_moderator_role(&ctx, &mut pool, guild_id).await?;
    // TODO: (?) Handle adding a log channel, in case we have errors we want to output
    setup_reaction_message(&ctx, &mut pool, guild_id, ticket_channel).await?;

    ctx.send_simple_message(messages::MSG_SETUP_SUCCESS).await?;

    sqlx::query!(
        "UPDATE servers SET setup_complete = true WHERE id = $1",
        guild_id.get() as i64
    )
    .execute(&mut *pool)
    .await?;

    Ok(())
}

async fn ask_for_ticket_channel_id(ctx: &Context<'_>) -> Result<ChannelId, Error> {
    ctx.send_simple_message(messages::MSG_SETUP_CHANNEL_ID)
        .await?;

    parse_channel_id_from_user_input(ctx).await
}

async fn setup_log_channel(ctx: &Context<'_>, pool: &mut PgConnection) -> Result<(), Error> {
    // Get the log channel ID
    ctx.send_simple_message("Please provide the channel ID to be used for logging")
        .await?;

    let log_channel = parse_channel_id_from_user_input(ctx).await?;

    // Set the log channel ID in the database
    sqlx::query!(
        "UPDATE servers SET log_channel_id = $1 WHERE id = $2",
        log_channel.get() as i64,
        ctx.guild_id().ok_or(messages::MSG_GUILD_FAIL)?.get() as i64
    )
    .execute(&mut *pool)
    .await?;

    Ok(())
}

async fn parse_channel_id_from_user_input(ctx: &Context<'_>) -> Result<ChannelId, Error> {
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

    Ok(channel_id.into())
}

async fn setup_request_channel(
    ctx: &Context<'_>,
    pool: &mut PgConnection,
    guild_id: GuildId,
    channel_id: ChannelId,
) -> Result<(), Error> {
    let Ok(channel) = ctx.http().get_channel(channel_id).await else {
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
        channel_id.get() as i64,
        guild_id.get() as i64
    )
    .execute(&mut *pool)
    .await?;

    Ok(())
}

/// Setup the helper role
///
/// Helpers are the people who can claim and see the tickets
/// They cannot however close the tickets (reserved for moderators)
async fn setup_helper_role(
    ctx: &Context<'_>,
    pool: &mut PgConnection,
    guild_id: GuildId,
) -> Result<(), Error> {
    let role = get_new_or_existing_role(ctx, guild_id, "Helper Role Setup", "Helper").await?;

    sqlx::query!(
        "UPDATE servers SET helper_role_id = $1 WHERE id = $2",
        role.get() as i64,
        guild_id.get() as i64
    )
    .execute(&mut *pool)
    .await?;

    Ok(())
}

/// Setup the moderator role
///
/// Moderators are the people who can see and close the tickets
/// They cannot claim the tickets (reserved for helpers)
async fn setup_moderator_role(
    ctx: &Context<'_>,
    pool: &mut PgConnection,
    guild_id: GuildId,
) -> Result<(), Error> {
    let role_id =
        get_new_or_existing_role(ctx, guild_id, "Moderator Role Setup", "Moderator").await?;

    sqlx::query!(
        "UPDATE servers SET moderator_role_id = $1 WHERE id = $2",
        role_id.get() as i64,
        guild_id.get() as i64
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
    guild_id: GuildId,
) -> Result<(), Error> {
    let unclaimed = create_server_category(ctx, guild_id, "Unclaimed Tickets").await?;
    let claimed = create_server_category(ctx, guild_id, "Claimed Tickets").await?;

    sqlx::query!(
        "UPDATE servers SET unclaimed_category_id = $1, claimed_category_id = $2 WHERE
        id = $3",
        unclaimed.id.get() as i64,
        claimed.id.get() as i64,
        guild_id.get() as i64
    )
    .execute(&mut *pool)
    .await?;

    Ok(())
}

/// Creates a server category
async fn create_server_category(
    ctx: &Context<'_>,
    guild_id: GuildId,
    name: &str,
) -> Result<GuildChannel, Error> {
    let builder = CreateChannel::new("")
        .name(name)
        .kind(ChannelType::Category);

    let category = ctx.http().create_channel(guild_id, &builder, None).await?;

    Ok(category)
}

/// Sets up the reaction message, sent in the ticket channel provided by the user
async fn setup_reaction_message(
    ctx: &Context<'_>,
    pool: &mut PgConnection,
    guild_id: GuildId,
    channel_id: ChannelId,
) -> Result<(), Error> {
    let embed = CreateEmbed::default_bot_embed(guild_id.to_partial_guild(ctx.http()).await?)
        .title("Open a ticket")
        .description("React to this message to open a ticket");

    let sent_message = channel_id
        .send_message(&ctx, CreateMessage::default().embed(embed))
        .await?;

    sqlx::query!(
        "UPDATE servers SET ticket_message_id = $1 WHERE id = $2",
        sent_message.id.get() as i64,
        guild_id.get() as i64
    )
    .execute(&mut *pool)
    .await?;

    sent_message
        .react(&ctx, ReactionType::Unicode(TICKET_EMOJI.into()))
        .await?;

    Ok(())
}
