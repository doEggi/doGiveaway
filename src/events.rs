use std::time::Duration;

use anyhow::{Error, Result};
use chrono::Timelike;
use poise::{
    serenity_prelude::{Context, FullEvent, Http, Reaction},
    FrameworkContext,
};
use tokio::time::sleep;

use crate::{
    giveaway::{Giveaway, GiveawayId},
    state::State,
};

pub async fn handle_reaction_add(reaction: &Reaction, state: State) {
    //
    let mut state = state.lock().await;
    let giveaway = if let Some(val) = state
        .giveaways
        .iter_mut()
        .filter(|val| val.message == reaction.message_id)
        .next()
    {
        val
    } else {
        return;
    };
    if !reaction.emoji.unicode_eq(&giveaway.emoji) {
        return;
    }
    giveaway.users.push(reaction.user_id.unwrap());
}

pub async fn handle_reaction_remove(reaction: &Reaction, state: State) {
    let mut state = state.lock().await;
    let giveaway = if let Some(val) = state
        .giveaways
        .iter_mut()
        .filter(|val| val.message == reaction.message_id)
        .next()
    {
        val
    } else {
        return;
    };
    if !reaction.emoji.unicode_eq(&giveaway.emoji) {
        return;
    }
    if let Some(index) = giveaway
        .users
        .iter()
        .position(|val| *val == reaction.user_id.unwrap())
    {
        giveaway.users.swap_remove(index);
    }
}

pub async fn handle_event(
    _context: &Context,
    event: &FullEvent,
    _framework_context: FrameworkContext<'_, State, Error>,
    state: &State,
) -> Result<()> {
    match event {
        FullEvent::ReactionAdd {
            add_reaction: reaction,
        } => handle_reaction_add(reaction, state.clone()).await,
        FullEvent::ReactionRemove {
            removed_reaction: reaction,
        } => handle_reaction_remove(reaction, state.clone()).await,
        _ => {}
    }
    //println!("Event: {}", event.snake_case_name());
    Ok(())
}

pub async fn handle_timeouts(state: State, http: impl AsRef<Http>) -> ! {
    println!("Started timeout handler");
    loop {
        let giveaways: Vec<Giveaway> = {
            let now = chrono::offset::Utc::now();
            let mut state = state.lock().await;
            let ids: Vec<GiveawayId> = state
                .giveaways
                .iter()
                .rev()
                .filter_map(|giveaway| match giveaway.time {
                    None => None,
                    Some(val) => {
                        if val <= now {
                            Some(giveaway.id)
                        } else {
                            None
                        }
                    }
                })
                .collect();

            ids.iter()
                .map(|id| {
                    let index = state
                        .giveaways
                        .iter()
                        .position(|val| val.id == *id)
                        .unwrap();
                    state.giveaways.swap_remove(index)
                })
                .collect()
        };

        for giveaway in giveaways {
            giveaway.finish(&http).await.unwrap();
        }

        let duration = Duration::from_secs((60 - chrono::offset::Local::now().second()).into());
        sleep(duration).await;
    }
}
