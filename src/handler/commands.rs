//! This module regroups the commands supported by the discord bot.

use crate::database::is_server_setup;

use super::{Context as MyContext, Data, Error as MyError};
use poise::{serenity_prelude::Error, Command, Context, CreateReply, ReplyHandle};

pub mod setup;
pub mod subject;

/// Get all the commands supported by the bot
pub fn get() -> Vec<Command<Data, super::Error>> {
    vec![
        setup::setup(),
        subject::add::add_slash(),
        subject::add::add_prefix(),
        subject::list::list(),
        subject::remove::remove(),
    ]
}

/// Helper function to check if the server is set up
async fn check_server_setup(ctx: MyContext<'_>) -> Result<bool, MyError> {
    let mut pool = ctx.data().pool.acquire().await?;
    let guild_id = ctx.guild_id().ok_or("Failed to get guild ID")?;
    Ok(is_server_setup(&mut pool, guild_id).await? == Some(true))
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
