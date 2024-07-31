use poise::{Framework, FrameworkOptions};

mod commands;
mod events;

// Types used by all command functions
pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, Data, Error>;

pub struct Data {
    pool: sqlx::Pool<sqlx::Postgres>,
}

impl Data {}

async fn on_error(error: poise::FrameworkError<'_, Data, Error>) {
    match error {
        poise::FrameworkError::Setup { error, .. } => panic!("Failed to start bot: {error:?}"),
        poise::FrameworkError::Command { error, ctx, .. } => {
            println!("Error in command `{}`: {:?}", ctx.command().name, error,);
        }
        error => {
            if let Err(e) = poise::builtins::on_error(error).await {
                println!("Error while handling error: {e}");
            }
        }
    }
}

fn get_functions() -> FrameworkOptions<Data, Error> {
    FrameworkOptions {
        commands: commands::get(),
        event_handler: |ctx, event, framework, data| {
            Box::pin(events::event_handler(ctx, event, framework, data))
        },
        prefix_options: poise::PrefixFrameworkOptions {
            prefix: Some("$".to_string()),
            edit_tracker: None,
            ..Default::default()
        },
        on_error: |error| Box::pin(on_error(error)),
        pre_command: |ctx| {
            Box::pin(async move {
                info!("Executing command {}...", ctx.command().qualified_name);
            })
        },
        post_command: |ctx| {
            Box::pin(async move {
                info!("Executed command {}", ctx.command().qualified_name);
            })
        },
        ..Default::default()
    }
}

pub fn get_framework(pool: sqlx::Pool<sqlx::Postgres>) -> poise::Framework<Data, Error> {
    Framework::builder()
        .options(get_functions())
        .setup(move |ctx, ready, framework| {
            Box::pin(async move {
                println!("Logged in as {}", ready.user.name);
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data { pool })
            })
        })
        .build()
}
