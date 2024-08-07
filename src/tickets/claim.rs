use crate::handler::{Context, Error};
use poise::serenity_prelude::{ChannelId, ChannelType, EditChannel};

pub async fn claim(ctx: &Context<'_>) -> Result<(), Error> {
    let mut pool = ctx.data().pool.acquire().await?;
    let guild_id = ctx.guild_id().ok_or("Not in a guild")?;

    let row = sqlx::query!(
        "SELECT helper_role_id FROM servers WHERE id = $1",
        guild_id.get() as i64
    )
    .fetch_optional(&mut *pool)
    .await?;

    let role_id = row
        .map(|row| row.helper_role_id)
        .ok_or("Failed to find the server")? as u64;

    // Cannot claim
    if !ctx.author().has_role(ctx.http(), guild_id, role_id).await? {
        return Ok(());
    }

    let channel = ctx.channel_id();

    let ticket_id = sqlx::query!(
        "SELECT ticket_id FROM tickets WHERE channel_id = $1",
        channel.get() as i64
    )
    .fetch_optional(&mut *pool)
    .await?;

    // Not a ticket channel
    if ticket_id.is_none() {
        return Ok(());
    }

    // Change category
    let category_channel_id: ChannelId = ChannelId::from(
        sqlx::query!(
            "SELECT claimed_category_id FROM servers WHERE id = $1",
            guild_id.get() as i64
        )
        .fetch_one(&mut *pool)
        .await?
        .claimed_category_id as u64,
    );

    let edit_channel = EditChannel::new()
        .kind(ChannelType::Text)
        .category(category_channel_id);

    ctx.channel_id().edit(ctx.http(), edit_channel).await?;

    Ok(())
}
