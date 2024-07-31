use poise::serenity_prelude::{
    self as serenity, CacheHttp, FullEvent, ReactionType, ReactionTypes,
};

use crate::tickets::TICKET_EMOJI;

use super::{Data, Error};

pub async fn event_handler(
    ctx: &serenity::Context,
    event: &serenity::FullEvent,
    _framework: poise::FrameworkContext<'_, Data, Error>,
    data: &Data,
) -> Result<(), Error> {
    match event {
        FullEvent::ReactionAdd { add_reaction } => {
            handle_reaction(ctx, add_reaction, data).await?;
        }
        _ => {}
    }
    Ok(())
}

async fn handle_reaction(
    ctx: &serenity::Context,
    reaction: &serenity::Reaction,
    data: &Data,
) -> Result<(), Error> {
    let user = reaction.user(ctx.http()).await?;
    let mut pool = data.pool.acquire().await?;

    if user.bot {
        return Ok(());
    }

    if reaction.emoji != ReactionType::Unicode(TICKET_EMOJI.to_string()) {
        return Ok(());
    }

    let row = sqlx::query!(
        "SELECT unclaimed_category_id FROM servers WHERE
        id = $1 AND 
        ticket_channel_id = $2 AND 
        ticket_message_id = $3",
        reaction.guild_id.unwrap().get() as i64,
        reaction.channel_id.get() as i64,
        reaction.message_id.get() as i64
    )
    .fetch_optional(&mut *pool)
    .await?;

    reaction.delete(ctx.http()).await?;

    Ok(())
}
