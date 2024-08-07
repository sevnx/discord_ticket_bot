//! This module regroups utilities linked to roles setup.

use std::time::Duration;

use poise::serenity_prelude::{
    CreateEmbed, CreateMessage, EditRole, GuildId, PartialGuild, ReactionType, RoleId,
};

use crate::{
    handler::{commands::SimpleMessage, Context, Error},
    helper::{embed::Custom, parser::parse_discord_mention},
};

pub async fn get_new_or_existing_role(
    ctx: &Context<'_>,
    guild: &PartialGuild,
    title: &str,
    role_name: &str,
) -> Result<RoleId, Error> {
    // Ask if the user wants to create a new role or use an existing one
    let embed = CreateEmbed::default()
        .title(title)
        .description("Do you want to create a new role or use an existing one?")
        .field("🆕", "Create a new role", true)
        .field("🔗", "Use an existing role", true);

    let reaction_new = ReactionType::Unicode("🆕".to_string());
    let reaction_existing = ReactionType::Unicode("🔗".to_string());

    let message = CreateMessage::default()
        .embed(embed)
        .reactions(vec![reaction_new.clone(), reaction_existing.clone()]);

    let sent_message = ctx.channel_id().send_message(ctx.http(), message).await?;

    let reaction = sent_message
        .await_reaction(ctx)
        .timeout(std::time::Duration::from_secs(60))
        .await
        .ok_or("Timed out waiting for reaction")?;

    let role_id = match reaction.emoji {
        ref emoji if emoji == &reaction_new => create_new_role(ctx, guild.id, role_name).await?,
        ref emoji if emoji == &reaction_existing => select_existing_role(ctx, guild).await?,
        _ => {
            return Err("Invalid reaction".into());
        }
    };

    Ok(role_id)
}

/// Create a new role for the helpers
async fn create_new_role(
    ctx: &Context<'_>,
    guild_id: GuildId,
    name: &str,
) -> Result<RoleId, Error> {
    let role = EditRole::default().name(name);
    let created_role = guild_id.create_role(ctx.http(), role).await?;

    Ok(created_role.id)
}

/// Select an existing role for the helpers
async fn select_existing_role(ctx: &Context<'_>, guild: &PartialGuild) -> Result<RoleId, Error> {
    // Ask the user to mention the role
    let embed = CreateEmbed::default_bot_embed(guild)
        .title("Select a role")
        .description("Please mention the role you want to use as the helper role");

    ctx.channel_id()
        .send_message(ctx.http(), CreateMessage::default().embed(embed))
        .await?;

    // Wait for the user to mention the role
    let Some(reply) = ctx
        .author()
        .await_reply(ctx)
        .timeout(Duration::from_secs(60))
        .await
    else {
        ctx.send_simple_message("Timed out waiting for reply")
            .await?;
        return Err("Timed out waiting for reply".into());
    };

    // Parse the role ID
    let role_id = parse_discord_mention(reply.content.trim())
        .ok_or("Invalid role ID")?
        .into();

    Ok(role_id)
}
