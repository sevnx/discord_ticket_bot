//! Setup command used to set up the bot in a server

use crate::handler::{
    commands::{check_server_setup, SimpleMessage},
    Context, Error,
};
use poise::{
    command,
    serenity_prelude::{ChannelId, CreateEmbed, CreateMessage, ReactionType, RoleId},
};
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
    const CONFIRM_MESSAGE: &str = "CONFIRM";

    let guild = ctx
        .guild_id()
        .ok_or("Failed to get guild ID")?
        .to_partial_guild(ctx.http())
        .await?;
    let mut pool = ctx.data().pool.acquire().await?;

    // Ask for reset confirmation

    ctx.send_simple_message(&format!(
        "Are you sure you want to reset the server? Type `{CONFIRM_MESSAGE}` to confirm"
    ))
    .await?;

    let reply = ctx
        .author()
        .await_reply(ctx)
        .await
        .ok_or("Timeout reached")?;

    if reply.content != CONFIRM_MESSAGE {
        ctx.send_simple_message("Reset cancelled").await?;
        return Ok(());
    }

    // TODO: Delete all tickets, subjects, logs etc.

    // Delete server from database and get server info for deletions
    let server_info = sqlx::query!(
        "DELETE FROM servers WHERE id = $1 RETURNING *",
        guild.id.get() as i64
    )
    .fetch_one(&mut *pool)
    .await?;

    // Roles
    if get_yes_no_answer(ctx, "Do you want to delete the helper role?").await? {
        let helper_role_id = RoleId::from(server_info.helper_role_id as u64);
        guild.id.delete_role(ctx.http(), helper_role_id).await?;
    }

    if get_yes_no_answer(ctx, "Do you want to delete the moderator role?").await? {
        let moderator_role_id = RoleId::from(server_info.moderator_role_id as u64);
        guild.id.delete_role(ctx.http(), moderator_role_id).await?;
    }

    // Categories
    if get_yes_no_answer(ctx, "Do you want to delete the unclaimed tickets category?").await? {
        let unclaimed_category_id = ChannelId::from(server_info.unclaimed_category_id as u64);
        unclaimed_category_id.delete(ctx.http()).await?;
    }

    if get_yes_no_answer(ctx, "Do you want to delete the claimed tickets category?").await? {
        let claimed_category_id = ChannelId::from(server_info.claimed_category_id as u64);
        claimed_category_id.delete(ctx.http()).await?;
    }

    // Channels aren't deleted because they are the ticket and log channels
    // (which are provided by the server administator therefore not subject to this reset)

    ctx.send_simple_message("Reset successful").await?;

    Ok(())
}

/// Helper function for y/n confirmation
async fn get_yes_no_answer(ctx: Context<'_>, question: &str) -> Result<bool, Error> {
    let embed = CreateEmbed::default()
        .title("Confirmation")
        .description(question)
        .field("✅", "Yes", true)
        .field("❌", "No", true);

    let reaction_yes = ReactionType::Unicode("✅".to_string());
    let reaction_no = ReactionType::Unicode("❌".to_string());

    let message = CreateMessage::default()
        .embed(embed)
        .reactions(vec![reaction_yes.clone(), reaction_no.clone()]);

    let sent_message = ctx.channel_id().send_message(ctx.http(), message).await?;

    let reaction = sent_message
        .await_reaction(ctx)
        .timeout(Duration::from_secs(60))
        .await
        .ok_or("Timed out waiting for reaction")?;

    Ok(reaction.emoji == reaction_yes)
}
