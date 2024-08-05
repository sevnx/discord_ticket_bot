use std::time::Duration;

use crate::{
    database::get_subjects,
    handler::{Data, Error},
    helper::fuzzy_match::fuzzy_match_subjects,
};
use poise::serenity_prelude::{
    CacheHttp, ChannelId, ChannelType, ComponentInteractionDataKind, Context, CreateActionRow,
    CreateChannel, CreateEmbed, CreateMessage, CreateSelectMenu, CreateSelectMenuKind,
    CreateSelectMenuOption, EditChannel, Http, Member, User,
};

use super::TICKET_EMOJI;

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

    // Send DM
    let cache_copy = ctx.http.clone();
    let user = member.user.clone();
    let channel_id = channel.id;

    tokio::spawn(async move {
        send_opened_ticket_dm(&cache_copy, &user, &channel_id)
            .await
            .unwrap_or_else(|e| error!("Failed to send DM: {}", e));
    });

    // Send message in channel
    // TODO: Change message to embed to make it look better
    let message = CreateMessage::new().content(format!(
        "Hello <@{}>, welcome to your ticket channel, please type out the subject of your ticket",
        member.user.id
    ));

    channel.send_message(ctx.http(), message).await?;

    // Wait for user input
    let subject = match channel
        .await_reply(ctx)
        .timeout(Duration::from_secs(60))
        .await
    {
        Some(reply) => reply.content,
        None => {
            // TODO: Close ticket
            // TODO: Send DM to user that the ticket has been closed
            return Ok(());
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

    let embed = CreateEmbed::default()
        .title("Select an option")
        .description("Please select the subject of your ticket")
        .color(0x00ff00);

    let message = CreateMessage::default()
        .embed(embed)
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

async fn send_opened_ticket_dm(
    http_cache: &Http,
    user: &User,
    channel_id: &ChannelId,
) -> Result<(), Error> {
    // TODO: Change message to embed to make it look better
    let dm_message = CreateMessage::new()
        .content(format!("Your ticket has been created : <#{}>", channel_id))
        .tts(false);

    user.dm(http_cache, dm_message).await?;

    Ok(())
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
