#![allow(clippy::cast_possible_wrap)]

use dotenv::dotenv;
use handler::get_framework;
use poise::serenity_prelude::{Client, GatewayIntents};
use std::env;

#[macro_use]
extern crate tracing;

// Crate modules
mod database;
mod handler;
mod helper;
mod logging;
mod tickets;

#[tokio::main]
async fn main() {
    logging::setup().unwrap_or_else(|error| panic!("Failed to set up logging: {error}"));

    dotenv().unwrap_or_else(|error| panic!("Failed to load .env file : {error}"));

    let db_pool = database::get_database_pool()
        .await
        .unwrap_or_else(|error| panic!("Failed to create database pool: {error}"));

    let discord_token = env::var("DISCORD_TOKEN")
        .unwrap_or_else(|error| panic!("Failed to get DISCORD_TOKEN from .env file : {error}"));

    let intents = GatewayIntents::GUILDS
        | GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::GUILD_MESSAGE_REACTIONS
        | GatewayIntents::MESSAGE_CONTENT;

    let mut client = Client::builder(discord_token, intents)
        .framework(get_framework(db_pool))
        .intents(intents)
        .await
        .unwrap_or_else(|error| panic!("Failed to create client: {error}"));

    if let Err(error) = client.start().await {
        error!("Client error: {error}");
    }

    info!("Client shutting down");
}
