use poise::serenity_prelude::{self as serenity, CacheHttp, Context, FullEvent, ReactionType};

use crate::tickets::{self, TICKET_EMOJI};

use super::{Data, Error};

pub async fn event_handler(
    ctx: &Context,
    event: &serenity::FullEvent,
    _: poise::FrameworkContext<'_, Data, Error>,
    data: &Data,
) -> Result<(), Error> {
    // TODO: Improve error handling
    if let FullEvent::ReactionAdd { add_reaction } = event {
        handle_reaction(ctx, add_reaction, data).await?;
    }
    Ok(())
}

async fn handle_reaction(
    ctx: &Context,
    reaction: &serenity::Reaction,
    data: &Data,
) -> Result<(), Error> {
    // TODO: Improve error handling
    let guild_id = reaction.guild_id.ok_or("Failed to get guild ID")?;
    let user_id = reaction.user_id.ok_or("Failed to get user ID")?;

    let member = guild_id.member(ctx, user_id).await?;

    let mut pool = data.pool.acquire().await?;

    if member.user.bot {
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
        guild_id.get() as i64,
        reaction.channel_id.get() as i64,
        reaction.message_id.get() as i64
    )
    .fetch_optional(&mut *pool)
    .await?;

    match row {
        Some(row) => {
            let unclaimed_id = row.unclaimed_category_id;
            tickets::create_ticket(ctx, data, &member, unclaimed_id as u64).await?;
        }
        None => {
            return Ok(());
        }
    }

    info!("Deleting reaction");
    reaction.delete(ctx.http()).await?;

    Ok(())
}
