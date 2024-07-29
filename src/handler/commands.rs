//! This module regroups the commands supported by the discord bot.

use poise::{serenity_prelude::Error, Command, Context, CreateReply, ReplyHandle};

use super::Data;

pub mod setup;
pub mod subject;

/// Get all the commands supported by the bot
pub fn get() -> Vec<Command<Data, super::Error>> {
    vec![
        setup::setup(),
        subject::add(),
        subject::remove(),
        subject::list(),
    ]
}

/// Helper trait to send simple messages (text only)
pub trait SimpleMessage<'a, E> {
    async fn send_simple_message(&self, text: impl Into<String>) -> Result<ReplyHandle<'a>, Error>;
}

impl<'a, U, E> SimpleMessage<'a, E> for Context<'a, U, E> {
    async fn send_simple_message(&self, text: impl Into<String>) -> Result<ReplyHandle<'a>, Error> {
        self.send(CreateReply::default().content(text.into()).reply(false))
            .await
    }
}
