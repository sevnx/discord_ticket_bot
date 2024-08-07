//! Setup command used to set up the bot in a server

use crate::{
    database::is_server_setup,
    handler::{commands::SimpleMessage, Context, Error},
    helper::{embed::Custom, parser::parse_discord_channel_id_url},
    tickets::TICKET_EMOJI,
};
use poise::{
    command,
    serenity_prelude::{
        model::channel, ChannelId, ChannelType, CreateChannel, CreateEmbed, CreateMessage,
        GuildChannel, GuildId, MessageId, ReactionType, RoleId,
    },
};
use roles::get_new_or_existing_role;
use sqlx::PgConnection;
use std::time::Duration;

mod roles;

/// Setup the bot in a server
#[command(
    slash_command,
    prefix_command,
    required_permissions = "ADMINISTRATOR",
    guild_only
)]
pub async fn setup(ctx: Context<'_>) -> Result<(), Error> {
    let mut guild_info = ServerSetupBuilder::new();

    let guild = ctx
        .guild_id()
        .ok_or("Failed to get guild ID")?
        .to_partial_guild(ctx.http())
        .await?;
    let mut pool = ctx.data().pool.acquire().await?;
    if is_server_setup(&mut pool, guild.id).await? {
        ctx.send_simple_message("The server is already set up")
            .await?;
        return Ok(());
    }

    info!("Setting up server {}", guild.name);

    // Guild ID
    guild_info.guild(guild.id);

    // Channels
    info!("Setting up ticket channels for {}", guild.name);
    guild_info.ticket_channel(get_ticket_channel_id(&ctx).await?);
    guild_info.log_channel(get_log_channel_id(&ctx).await?);

    // Categories
    info!("Setting up categories for {}", guild.name);
    guild_info.unclaimed_category(
        create_server_category(&ctx, guild.id, "Unclaimed Tickets")
            .await?
            .id,
    );
    guild_info.claimed_category(
        create_server_category(&ctx, guild.id, "Claimed Tickets")
            .await?
            .id,
    );

    // Roles
    info!("Setting up roles for {}", guild.name);
    guild_info.helper_role(get_new_or_existing_role(&ctx, &guild, "Helper Role", "Helper").await?);

    guild_info.moderator_role(
        get_new_or_existing_role(&ctx, &guild, "Moderator Role", "Moderator").await?,
    );

    // Save information about the ticket channel
    info!("Saving server setup data for {}", guild.name);
    guild_info.build()?.setup(&ctx).await
}

/// Creates a server category
async fn create_server_category(
    ctx: &Context<'_>,
    guild_id: GuildId,
    name: &str,
) -> Result<GuildChannel, Error> {
    let builder = CreateChannel::new("")
        .name(name)
        .kind(ChannelType::Category);

    let category = ctx.http().create_channel(guild_id, &builder, None).await?;

    Ok(category)
}

async fn get_ticket_channel_id(ctx: &Context<'_>) -> Result<ChannelId, Error> {
    ctx.send_simple_message("Please provide the ticket channel ID")
        .await?;

    let channel = parse_channel_id_from_user_input(ctx).await?;

    if !is_guild_text_channel(ctx, channel).await? {
        ctx.send_simple_message("The channel must be a text channel")
            .await?;
        return Err("The channel must be a text channel".into());
    }

    Ok(channel)
}

async fn get_log_channel_id(ctx: &Context<'_>) -> Result<ChannelId, Error> {
    ctx.send_simple_message("Please provide the log channel ID")
        .await?;

    let channel = parse_channel_id_from_user_input(ctx).await?;

    if !is_guild_text_channel(ctx, channel).await? {
        ctx.send_simple_message("The channel must be a text channel")
            .await?;
        return Err("The channel must be a text channel".into());
    }

    Ok(channel)
}

async fn parse_channel_id_from_user_input(ctx: &Context<'_>) -> Result<ChannelId, Error> {
    let Some(reply) = ctx
        .author()
        .await_reply(ctx)
        .timeout(Duration::from_secs(60))
        .await
    else {
        ctx.send_simple_message("Timeout reached").await?;
        return Err("Timeout reached".into());
    };

    let content = reply.content.trim();

    let channel_id: u64 = if let Some(channel_id) = parse_discord_channel_id_url(content) {
        channel_id
    } else {
        ctx.send_simple_message("Timeout reached").await?;
        return Err("Timeout reached".into());
    };

    Ok(channel_id.into())
}

async fn is_guild_text_channel(ctx: &Context<'_>, channel_id: ChannelId) -> Result<bool, Error> {
    let Ok(channel) = ctx.http().get_channel(channel_id).await else {
        ctx.send_simple_message("Failed to get channel").await?;
        return Err("Failed to get channel".into());
    };

    if let channel::Channel::Guild(channel) = channel {
        if channel.kind != ChannelType::Text {
            return Ok(false);
        }
    }

    Ok(true)
}

#[derive(Default)]
struct ServerSetupBuilder {
    guild: Option<GuildId>,
    ticket_channel: Option<ChannelId>,
    unclaimed_category: Option<ChannelId>,
    claimed_category: Option<ChannelId>,
    log_channel: Option<ChannelId>,
    helper_role: Option<RoleId>,
    moderator_role: Option<RoleId>,
}

impl ServerSetupBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn guild(&mut self, id: GuildId) -> &Self {
        self.guild = Some(id);
        self
    }

    pub fn ticket_channel(&mut self, id: ChannelId) -> &Self {
        self.ticket_channel = Some(id);
        self
    }

    pub fn unclaimed_category(&mut self, id: ChannelId) -> &Self {
        self.unclaimed_category = Some(id);
        self
    }

    pub fn claimed_category(&mut self, id: ChannelId) -> &Self {
        self.claimed_category = Some(id);
        self
    }

    pub fn log_channel(&mut self, id: ChannelId) -> &Self {
        self.log_channel = Some(id);
        self
    }

    pub fn helper_role(&mut self, id: RoleId) -> &Self {
        self.helper_role = Some(id);
        self
    }

    pub fn moderator_role(&mut self, id: RoleId) -> &Self {
        self.moderator_role = Some(id);
        self
    }

    /// Builds the server setup data
    ///
    /// It will return an error if any of the required fields are missing
    pub fn build(self) -> Result<ServerSetupData, String> {
        Ok(ServerSetupData {
            guild: self.guild.ok_or("Guild ID is required")?,
            ticket_channel: self.ticket_channel.ok_or("Ticket channel ID is required")?,
            unclaimed_category: self
                .unclaimed_category
                .ok_or("Unclaimed category ID is required")?,
            claimed_category: self
                .claimed_category
                .ok_or("Claimed category ID is required")?,
            log_channel: self.log_channel.ok_or("Log channel ID is required")?,
            helper_role: self.helper_role.ok_or("Helper role ID is required")?,
            moderator_role: self.moderator_role.ok_or("Moderator role ID is required")?,
        })
    }
}

struct ServerSetupData {
    /// The guild ID
    guild: GuildId,
    /// The channel where the bot will send the ticket message and listen for reactions
    ticket_channel: ChannelId,
    /// The category where the unclaimed tickets will be created
    unclaimed_category: ChannelId,
    /// The category where the claimed tickets will be placed
    claimed_category: ChannelId,
    /// The channel where the logs of the bot will be sent
    log_channel: ChannelId,
    /// The role ID of the helper role
    helper_role: RoleId,
    /// The role ID of the moderator role
    moderator_role: RoleId,
}

impl ServerSetupData {
    /// Sets up the server with the provided data
    ///
    /// It will create the ticket message and save the data to the database
    pub async fn setup(&self, ctx: &Context<'_>) -> Result<(), Error> {
        let message_id = self.setup_reaction_message(ctx).await?;

        let mut pool = ctx.data().pool.acquire().await?;

        self.save(&mut pool, message_id).await
    }

    async fn setup_reaction_message(&self, ctx: &Context<'_>) -> Result<MessageId, Error> {
        let guild = self.guild.to_partial_guild(ctx.http()).await?;

        let embed = CreateEmbed::default_bot_embed(&guild)
            .title("Open a ticket")
            .description("React to this message to open a ticket");

        let message = CreateMessage::default()
            .embed(embed)
            .reactions(vec![ReactionType::Unicode(TICKET_EMOJI.to_string())]);

        let sent_message = self
            .ticket_channel
            .send_message(ctx.http(), message)
            .await?;

        Ok(sent_message.id)
    }

    /// Save the server setup data to the database
    async fn save(
        &self,
        conn: &mut PgConnection,
        ticket_message_id: MessageId,
    ) -> Result<(), Error> {
        sqlx::query!(
            "INSERT INTO servers (
                id,
                ticket_channel_id,
                unclaimed_category_id,
                claimed_category_id,
                ticket_message_id,
                log_channel_id,
                helper_role_id,
                moderator_role_id
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
            self.guild.get() as i64,
            self.ticket_channel.get() as i64,
            self.unclaimed_category.get() as i64,
            self.claimed_category.get() as i64,
            ticket_message_id.get() as i64,
            self.log_channel.get() as i64,
            self.helper_role.get() as i64,
            self.moderator_role.get() as i64
        )
        .execute(conn)
        .await?;

        Ok(())
    }
}
