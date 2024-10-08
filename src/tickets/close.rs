use poise::serenity_prelude::{CreateEmbed, CreateMessage, Http, PartialGuild, UserId};

use crate::{
    handler::{Context, Error},
    helper::embed::Custom,
};

pub async fn close(ctx: &Context<'_>) -> Result<(), Error> {
    let guild = ctx
        .guild_id()
        .ok_or("Not in a guild")?
        .to_partial_guild(&ctx.http())
        .await?;

    let mut pool = ctx.data().pool.acquire().await?;

    let channel = ctx.channel_id();

    let Some(ticket) = sqlx::query!(
        "SELECT ticket_id, author_id FROM tickets WHERE channel_id = $1",
        channel.get() as i64
    )
    .fetch_optional(&mut *pool)
    .await?
    else {
        warn!("Tried to close a non-ticket channel");
        return Ok(());
    };

    // Check if the user is the author of the ticket
    if ctx.author().id != UserId::from(ticket.author_id as u64) {
        warn!("Tried to close a ticket that doesn't belong to them");
        return Ok(());
    }

    // TODO: Check if the user is a moderator (helpers can't close tickets)

    // TODO: Log the closing of the ticket

    send_closed_ticket_dm(
        UserId::from(ticket.author_id as u64),
        &guild,
        ctx.http(),
        "Ticket closed",
    )
    .await?;

    // Delete the channel
    ctx.channel_id().delete(&ctx.http()).await?;

    Ok(())
}

pub async fn send_closed_ticket_dm(
    user: UserId,
    guild: &PartialGuild,
    http_cache: &Http,
    reason: &str,
) -> Result<(), Error> {
    let embed = CreateEmbed::default_bot_embed(guild)
        .title("Ticket Closed")
        .field("Reason", reason, false);

    user.dm(http_cache, CreateMessage::default().embed(embed))
        .await?;

    Ok(())
}
