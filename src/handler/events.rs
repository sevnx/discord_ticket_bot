use poise::serenity_prelude::{self as serenity, CacheHttp, Context, FullEvent, ReactionType};

use crate::tickets::{self, TICKET_EMOJI};

use super::{Data, Error};

pub async fn event_handler(
    ctx: &Context,
    event: &serenity::FullEvent,
    _: poise::FrameworkContext<'_, Data, Error>,
    data: &Data,
) -> Result<(), Error> {
    match event {
        // TODO: (?) Handle other events (like `Ready` etc.)
        FullEvent::ReactionAdd { add_reaction } => {
            handle_reaction(ctx, add_reaction, data).await?;
        }
        _ => {}
    }
    Ok(())
}

async fn handle_reaction(
    ctx: &Context,
    reaction: &serenity::Reaction,
    data: &Data,
) -> Result<(), Error> {
    // TODO: Improve error handling
    let member = reaction
        .guild_id
        .unwrap()
        .member(ctx, reaction.user_id.unwrap())
        .await?;

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
        reaction.guild_id.unwrap().get() as i64,
        reaction.channel_id.get() as i64,
        reaction.message_id.get() as i64
    )
    .fetch_optional(&mut *pool)
    .await?;

    match row {
        Some(row) => {
            let unclaimed_id = row.unclaimed_category_id.unwrap();
            tickets::create::create(&ctx, data, &member, unclaimed_id as u64).await?;
        }
        None => {
            return Ok(());
        }
    }

    reaction.delete(ctx.http()).await?;

    Ok(())
}
