mod commands;
mod events;
mod giveaway;
mod state;

use anyhow::{Error, Result};
use commands::{cancel, clear, clearuser, create, finish, info};
use events::{handle_event, handle_timeouts};
use giveaway::Giveaway;
use poise::{
    serenity_prelude::{ClientBuilder, GatewayIntents},
    FrameworkError, FrameworkOptions,
};
use state::{InnerState, State};
use std::{
    fs::File,
    io::{Read, Write},
    sync::Arc,
};
use tokio::sync::Mutex;

#[tokio::main]
async fn main() -> Result<()> {
    let giveaways: Vec<Giveaway> = {
        let mut file = match File::open("state.toml") {
            Ok(val) => val,
            _ => File::create("state.toml")?,
        };
        let mut data = String::new();
        file.read_to_string(&mut data)?;
        toml::from_str(&data)?
    };

    let options: FrameworkOptions<State, Error> = poise::FrameworkOptions {
        commands: vec![create(), finish(), cancel(), clear(), clearuser(), info()],
        on_error: |error: FrameworkError<'_, State, Error>| {
            Box::pin(async move {
                match error {
                    FrameworkError::Command { error, ctx, .. } => {
                        _ = ctx.defer_ephemeral().await;
                        _ = ctx.say(format!("Fehler: {}", error)).await;
                        println!("Error: {:?}", error);
                    }
                    _ => println!("Unhandled error!"),
                }
            })
        },
        event_handler: (|ctx, event, framework, data| {
            Box::pin(async move { handle_event(ctx, event, framework, data).await })
        }),
        ..Default::default()
    };
    let state = Arc::new(Mutex::new(InnerState { giveaways }));
    let state_cpy = state.clone();

    let framework = poise::Framework::builder()
        .setup(move |ctx, ready, framework| {
            let http = ctx.http.clone();
            Box::pin(async move {
                println!("Logged in as {}", ready.user.name);
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                let state = state_cpy;
                let state2 = state.clone();
                tokio::spawn(async move {
                    handle_timeouts(state2, http).await;
                });
                Ok(state)
            })
        })
        .options(options)
        .build();

    let mut client = ClientBuilder::new(include_str!("../token"), GatewayIntents::non_privileged())
        .framework(framework)
        .await?;

    tokio::spawn(async move {
        if let Err(err) = client.start().await {
            println!("Fatal error: {:?}", err);
        }
    });

    tokio::signal::ctrl_c().await?;
    let state = state.lock().await;
    let mut file = File::create("state.toml")?;
    write!(file, "{}", toml::to_string(&state.giveaways)?)?;
    println!("State saved!");
    Ok(())
}
