//! This module regroups the commands supported by the discord bot.

use poise::{serenity_prelude::Error, Context, CreateReply, ReplyHandle};

pub mod setup;

/// Helper trait to send simple messages (text only)
pub trait SimpleMessage<'a, E> {
    async fn send_simple_message(&self, text: &str) -> Result<ReplyHandle<'a>, Error>;
}

impl<'a, U, E> SimpleMessage<'a, E> for Context<'a, U, E> {
    async fn send_simple_message(&self, text: &str) -> Result<ReplyHandle<'a>, Error> {
        self.send(CreateReply::default().content(text)).await
    }
}
