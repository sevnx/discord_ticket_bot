use poise::{
    serenity_prelude::{async_trait, Context, EventHandler, Message},
    Framework, FrameworkOptions, PrefixFrameworkOptions,
};

// Crate modules
mod commands;
mod events;
