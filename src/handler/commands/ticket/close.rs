use crate::handler::{commands::check_server_setup, Context, Error};
use poise::{
    command,
    serenity_prelude::{CreateMessage, UserId},
};

/// Closes a ticket
#[command(
    slash_command,
    prefix_command,
    required_permissions = "MANAGE_CHANNELS",
    check = "check_server_setup",
    guild_only
)]
pub async fn close(ctx: Context<'_>) -> Result<(), Error> {
    let mut pool = ctx.data().pool.acquire().await?;

    let channel = ctx.channel_id();

    let Some(ticket) = sqlx::query!(
        "SELECT ticket_id, author_id FROM tickets WHERE channel_id = $1",
        channel.get() as i64
    )
    .fetch_optional(&mut *pool)
    .await?
    else {
        return Ok(());
    };

    if ctx.author().id != UserId::from(ticket.author_id as u64) {
        return Ok(());
    }

    // TODO: Log the closing of the ticket

    // Send message to the user that the ticket has been closed
    let user = UserId::from(ticket.author_id as u64);

    let message = CreateMessage::new().content("Your ticket has been closed");
    user.dm(ctx.http(), message).await?;

    // Delete the channel
    ctx.channel_id().delete(&ctx.http()).await?;

    Ok(())
}
