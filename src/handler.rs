use serenity::{all::{Context, EventHandler, Message}, async_trait};

// Crate modules
mod events;
mod commands;

/// Handler for the bot
pub struct DesQuestionHandler;

#[async_trait]
impl EventHandler for DesQuestionHandler {
    async fn message(&self, ctx: Context, msg: Message) {
        handle_new_message(&ctx, &msg).await;
    }
}

/// Example function
async fn handle_new_message(_: &Context, msg: &Message) {
    info!(
        "New message received: content: '{}', author: '{}', channel: '{}', guild: '{}'",
        msg.content,
        msg.author.name,
        msg.channel_id,
        msg.guild_id.map_or("DM".to_string(), |id| id.to_string())
    );
}
