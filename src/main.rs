use std::env;
use dotenv::dotenv;
use handler::DesQuestionHandler;
use logging::setup_logging;
use serenity::all::GatewayIntents;

#[macro_use]
extern crate tracing;

// Crate modules
mod logging;
mod handler;

#[tokio::main]
async fn main() {
    setup_logging().unwrap_or_else(|error| panic!("Failed to setup logging : {error}"));

    dotenv().unwrap_or_else(|error| panic!("Failed to load .env file : {error}"));

    let token = env::var("DISCORD_TOKEN")
        .unwrap_or_else(|error| panic!("Failed to get DISCORD_TOKEN from .env file : {error}"));

    let intents = GatewayIntents::GUILDS
        | GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::GUILD_MESSAGE_REACTIONS;

    let mut client = serenity::Client::builder(token, intents)
        .event_handler(DesQuestionHandler)
        .intents(intents)
        .await
        .expect("Failed to create client");

    if let Err(error) = client.start().await {
        error!("Client error: {error}");
    }

    info!("Client shutting down");
}
