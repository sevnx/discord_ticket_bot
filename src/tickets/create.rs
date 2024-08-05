use std::time::Duration;

use crate::{
    database::get_subjects,
    handler::{Data, Error},
    helper::{embed::CustomEmbed, fuzzy_match::fuzzy_match_subjects},
};
use poise::serenity_prelude::{
    CacheHttp, ChannelId, ChannelType, ComponentInteractionDataKind, Context, CreateActionRow,
    CreateChannel, CreateEmbed, CreateEmbedFooter, CreateMessage, CreateSelectMenu,
    CreateSelectMenuKind, CreateSelectMenuOption, EditChannel, GuildId, Http, Member, Mentionable,
};

use super::{close::send_closed_ticket_dm, TICKET_EMOJI};

/// Handles the creation of a ticket
/// It creates a new channel in the unclaimed category
/// and sends a DM to the user with the channel id
pub async fn create(
    ctx: &Context,
    data: &Data,
    member: &Member,
    unclaimed_category_id: u64,
) -> Result<(), Error> {
    // TODO: Improve error handling
    let guild = member.guild_id;
    let mut pool = data.pool.acquire().await?;

    // Create channel
    let channel_builder = CreateChannel::new(temp_ticket_channel_name(member))
        .category(unclaimed_category_id)
        .position(0)
        .topic("Ticket channel")
        .kind(ChannelType::Text);

    let mut channel = guild.create_channel(ctx.http(), channel_builder).await?;

    // Send DM to the user in a separate task to avoid blocking
    let cache_copy = ctx.http.clone();
    let user = member.user.clone();
    let channel_id = channel.id;
    let guild_id = guild.clone();
    tokio::spawn(async move {
        let message = get_open_ticket_dm(&guild_id, &channel_id, &cache_copy).await;
        user.dm(cache_copy, message).await.unwrap_or_else(|e| {
            panic!("Failed to send DM to user: {}", e);
        });
    });

    // Send message in channel
    channel
        .send_message(
            ctx.http(),
            get_open_ticket_message(member, ctx.http()).await,
        )
        .await?;

    // Wait for user input
    // TODO: Make timeout configurable
    let subject = match channel
        .await_reply(ctx)
        .timeout(Duration::from_secs(60))
        .await
    {
        Some(reply) => reply.content,
        None => {
            return handle_timeout(&channel.id, member, &guild, ctx.http()).await;
        }
    };

    // Fuzzy match subjects
    let subjects = get_subjects(&mut *pool, guild).await?;
    let mut fuzzy_result = fuzzy_match_subjects(&subjects, &subject, 5);

    // Add default subject
    fuzzy_result.push(crate::database::Subject {
        id: None,
        name: "Other".to_string(),
    });

    // Send select menu
    let select_options: Vec<CreateSelectMenuOption> = fuzzy_result
        .iter()
        .enumerate()
        .map(|(i, subject)| CreateSelectMenuOption::new(subject.name.clone(), i.to_string()))
        .collect();

    let select_menu = CreateSelectMenu::new(
        "option_select",
        CreateSelectMenuKind::String {
            options: select_options,
        },
    );

    let message = CreateMessage::default()
        .embed(
            CreateEmbed::default_bot_embed(guild.to_partial_guild(ctx.http()).await?)
                .title("Select an option")
                .description("Please select the subject of your ticket"),
        )
        .components(vec![CreateActionRow::SelectMenu(select_menu)]);

    let sent = channel.send_message(ctx.http(), message).await?;

    // Wait for user input
    let subject = match sent
        .await_component_interaction(ctx)
        .timeout(Duration::from_secs(60))
        .await
    {
        Some(component) => match component.data.kind {
            ComponentInteractionDataKind::StringSelect { values } => {
                assert!(values.len() == 1);
                let index = values[0].parse::<usize>().unwrap();
                fuzzy_result.remove(index)
            }
            _ => return Ok(()),
        },
        None => return Ok(()),
    };

    // Update channel name
    let edit_channel = EditChannel::default().name(format!("{}-{}", TICKET_EMOJI, subject.name));

    channel.edit(ctx.http(), edit_channel).await?;

    // Add ticket to database
    sqlx::query!(
        "INSERT INTO tickets (channel_id, server_id, subject_id, author_id) VALUES ($1, $2, $3, $4)",
        channel.id.get() as i64,
        guild.get() as i64,
        subject.id.and_then(|id| Some(id as i64)),
        member.user.id.get() as i64
    ).execute(&mut *pool).await?;

    let message =
        CreateMessage::new().content(format!("Ticket created with subject: {}", subject.name));

    channel.send_message(ctx.http(), message).await?;

    Ok(())
}

/// Handles the timeout for the ticket creation
async fn handle_timeout(
    channel: &ChannelId,
    member: &Member,
    guild: &GuildId,
    http: &Http,
) -> Result<(), Error> {
    // Delete ticket channel
    channel.delete(http).await?;

    // Send DM to user
    send_closed_ticket_dm(
        member.user.id,
        guild.clone(),
        http,
        "Ticket creation timed out",
    )
    .await?;

    Ok(())
}

/// Returns an embed message to be sent to the user in DM when the person opens a ticket
async fn get_open_ticket_dm(
    guild_id: &GuildId,
    channel_id: &ChannelId,
    http: &Http,
) -> CreateMessage {
    let guild = guild_id.to_partial_guild(http).await.unwrap();

    let embed = CreateEmbed::default_bot_embed(guild)
        .title("Ticket Created")
        .field("Ticket Channel", format!("<#{}>", channel_id), false)
        .field(
            "Next Steps",
            "Please visit the ticket channel and provide details about your question or issue.",
            false,
        )
        .footer(CreateEmbedFooter::new(
            "To close the ticket, type `$close` in the ticket channel",
        ));

    CreateMessage::new().embed(embed)
}

/// Returns an embed message to be sent to the user in the ticket channel when the ticket is opened
async fn get_open_ticket_message(member: &Member, http: &Http) -> CreateMessage {
    let guild = member.guild_id.to_partial_guild(http).await.unwrap();

    let embed = CreateEmbed::default_bot_embed(guild)
        .title("Ticket Created")
        .description(format!(
            "Hello {} welcome to your ticket channel.",
            member.mention()
        ))
        .field(
            "Next Steps",
            "1. Type out the subject of your ticket, and choose the appropriate subject\n
            2. Provide a detailed description of your question or issue with any relevant information
            (e.g. screenshots, logs, etc.)\n
            3. Wait until the ticket is claimed by a helper",
            false,
        )
        .footer(CreateEmbedFooter::new(
            "To close the ticket, type `$close` in the ticket channel",
        ));

    CreateMessage::new().embed(embed)
}

/// Returns the name of the temporary name for the newly created ticket channel
/// It is temporary as the name will be changed based on the user's input
pub fn temp_ticket_channel_name(member: &Member) -> String {
    let current_time = chrono::Utc::now();
    format!(
        "{}-{}-{}",
        TICKET_EMOJI,
        member.display_name(),
        current_time.format("%d%m%Y%H%M%S")
    )
}
