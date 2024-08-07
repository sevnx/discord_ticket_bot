//! This module contains utilities for discord embed messages

use chrono::Utc;
use poise::serenity_prelude::{CreateEmbed, CreateEmbedAuthor, PartialGuild};

const DEFAULT_COLOR: u32 = 0x58_65F2;

/// Custom trait for embeds
pub trait Custom {
    /// Returns the embed builder with defaults (in the context of this bot)
    /// Defaults include
    /// - Author as the guild (name + icon)
    /// - Color as the default color
    /// - Timestamp as the current time
    fn default_bot_embed(guild: &PartialGuild) -> CreateEmbed;
}

/// Default implementation for `CreateEmbed`
impl Custom for CreateEmbed {
    fn default_bot_embed(guild: &PartialGuild) -> CreateEmbed {
        let icon = guild.icon_url().unwrap_or_else(|| {
            "https://upload.wikimedia.org/wikipedia/commons/4/48/BLANK_ICON.png".to_string()
        });
        let author = CreateEmbedAuthor::new(guild.name.clone()).icon_url(icon);

        Self::default()
            .author(author)
            .color(DEFAULT_COLOR)
            .timestamp(Utc::now())
    }
}
