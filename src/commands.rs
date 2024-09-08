use anyhow::{bail, Context as ctx, Error, Result};
use chrono::{DateTime, Timelike};
use poise::{
    serenity_prelude::{
        CreateChannel, CreateMessage, GetMessages, GuildChannel, Permissions, UserId,
    },
    Context,
};
use std::num::NonZero;

use crate::{
    giveaway::{Giveaway, GiveawayId},
    state::State,
};

#[poise::command(slash_command)]
pub async fn create(
    context: Context<'_, State, Error>,
    #[description = "Der Titel (Sollte den Preis enthalten)"] title: String,
    #[description = "Die Beschreibung"] description: String,
    #[description = "Der Kanal in dem das Giveaway stadtfindet"] channel: GuildChannel,
    #[description = "Die Anzahl der Gewinner"] winner_count: Option<u16>,
    #[description = "Das Emoji mit dem reagiert werden soll (Standard: 👍)"] emoji: Option<String>,
    #[description = "Ein UNIX Zeitstempel, wann das Giveaway enden soll"] time: Option<i64>,
) -> Result<()> {
    if !channel.is_text_based() {
        bail!("Der angegebene Kanal ist kein Textkanal")
    }
    let guild = channel.guild_id;
    let channel = channel.id;
    let time = match time {
        None => None,
        Some(val) => Some(
            DateTime::from_timestamp(val, 0)
                .context("Fehler beim erkennen des Zeitstempels")?
                .with_second(0)
                .unwrap(),
        ),
    };
    let emoji = emoji.unwrap_or("👍".to_string());
    if emoji.chars().count() != 1 {
        bail!("Emoji hat ungültiges Format")
    }
    let winner_count: NonZero<u16> = winner_count
        .unwrap_or(1)
        .try_into()
        .context("Die Anzahl der Gewinner darf nicht 0 sein")?;

    let mut message = format!("# {}\n\n{}", title, description);
    let used_ids: Vec<GiveawayId> = {
        let state = context.data().lock().await;
        state.giveaways.iter().map(|giveaway| giveaway.id).collect()
    };
    let mut id = GiveawayId::new();
    while used_ids.contains(&id) {
        //  This will never happen
        //  Better not testing my luck anyways
        id = GiveawayId::new();
    }

    if let Some(time) = time {
        message.push_str(&format!("\n\n{} UTC", time.format("%d.%m.%Y %H:%M")))
    }
    message.push_str(&format!(
        "\n\nReagiere mit {} um teilzunehmen.\nEs gibt {} Gewinner!",
        emoji, winner_count
    ));
    message.push_str(&format!("\n||{}||", id.0.get()));
    let message = channel
        .send_message(context.http(), CreateMessage::new().content(message))
        .await?
        .id;
    let giveaway = Giveaway {
        title,
        message,
        channel,
        emoji,
        guild,
        id,
        time,
        winner_count,
        users: Vec::new(),
    };
    //  TODO: Check for before or after...
    context.defer_ephemeral().await?;
    context.say("Giveaway erstellt!").await?;

    {
        let mut state = context.data().lock().await;
        state.giveaways.push(giveaway);
    }
    Ok(())
}

#[poise::command(slash_command)]
pub async fn finish(
    context: Context<'_, State, Error>,
    #[description = "Die ID vom Giveaway"] id: u32,
) -> Result<()> {
    let id = GiveawayId(id.try_into().context("Die ID darf nicht 0 sein")?);
    let mut state = context.data().lock().await;
    let index = state.giveaways.iter().position(|val| val.id == id);
    if let None = index {
        bail!("Kein Giveaway gefunden")
    } else if state.giveaways.get(index.unwrap()).unwrap().guild != context.guild_id().unwrap() {
        bail!(
            "Dieser befehl muss auf dem Server ausgeführt werden, auf dem das Giveaway stattfindet"
        )
    }
    let giveaway = state.giveaways.swap_remove(index.unwrap());
    giveaway.finish(context.http()).await?;
    context.defer_ephemeral().await?;
    context.say("Giveaway fertig!").await?;
    Ok(())
}

#[poise::command(slash_command)]
pub async fn cancel(
    context: Context<'_, State, Error>,
    #[description = "Die ID vom Giveaway"] id: u32,
) -> Result<()> {
    let id = GiveawayId(id.try_into().context("Die ID darf nicht 0 sein")?);
    let mut state = context.data().lock().await;
    let index = state.giveaways.iter().position(|val| val.id == id);
    if let None = index {
        bail!("Kein Giveaway gefunden")
    } else if state.giveaways.get(index.unwrap()).unwrap().guild != context.guild_id().unwrap() {
        bail!(
            "Dieser befehl muss auf dem Server ausgeführt werden, auf dem das Giveaway stattfindet"
        )
    }
    let giveaway = state.giveaways.swap_remove(index.unwrap());
    giveaway
        .cancel(context.http(), &context.author().id)
        .await?;
    context.defer_ephemeral().await?;
    context.say("Giveaway abgebrochen!").await?;
    Ok(())
}

#[poise::command(slash_command)]
/// Leert den aktuellen Kanal
pub async fn clear(context: Context<'_, State, Error>) -> Result<()> {
    let channel = context.guild_channel().await.context("Kein Server Kanal")?;
    let name = channel.name.clone();
    let position = channel.position;
    let kind = channel.kind;
    let nsfw = channel.nsfw;
    let p_overrides = channel.permission_overwrites.clone();
    let parent = channel.parent_id;
    let topic = channel.topic.clone();
    let rate_limit = channel.rate_limit_per_user;
    context.say("Done").await?;
    channel.delete(context.http()).await?;

    let mut builder = CreateChannel::new(name)
        .kind(kind)
        .position(position)
        .nsfw(nsfw)
        .permissions(p_overrides);
    if let Some(parent) = parent {
        builder = builder.category(parent);
    }
    if let Some(topic) = topic {
        builder = builder.topic(topic);
    }
    if let Some(rate_limit) = rate_limit {
        builder = builder.rate_limit_per_user(rate_limit);
    }

    let channel = context
        .guild_id()
        .context("Kein Server Kanal")?
        .create_channel(context.http(), builder)
        .await?;
    channel
        .send_message(
            context.http(),
            CreateMessage::new()
                .content("Kanal geleert!\nAlle Einladungen in diesen Kanal sind nun ungültig..."),
        )
        .await?;
    Ok(())
}

#[poise::command(slash_command)]
/// Entfernt Nachrichten eines Nutzers
pub async fn clearuser(
    context: Context<'_, State, Error>,
    #[description = "Der Nutzer dessen Nachrichten enfernt werden"] user: UserId,
    #[description = r#"True, wenn Nachrichten des gesamten Servers entfernt \
werden sollen, false nur für diesen Kanal"#]
    everything: Option<bool>,
) -> Result<()> {
    let everything = everything.unwrap_or(false);
    context.defer_ephemeral().await?;
    if everything {
        let channels = context.guild_id().unwrap().channels(context.http()).await?;
        for (channel, _) in channels {
            let messages = channel.messages(context.http(), GetMessages::new()).await?;
            for message in messages {
                if message.author.id == user {
                    message.delete(context.http()).await?;
                }
            }
        }
        context
            .say("Alle Nachrichten des Nutzers auf dem Server entfernt")
            .await?;
    } else {
        let channel = context.channel_id();
        let messages = channel.messages(context.http(), GetMessages::new()).await?;
        for message in messages {
            if message.author.id == user {
                message.delete(context.http()).await?;
            }
        }
        context
            .say("Alle Nachrichten des Nutzers in diesem Kanal entfernt")
            .await?;
    }
    Ok(())
}

pub async fn check_permission(context: Context<'_, State, Error>) -> Result<bool> {
    Ok(match context.author_member().await {
        Some(val) => {
            let permissions = val.permissions(context.cache())?;
            //  Im not sure if ADMINISTRATOR overrides everything else here...
            permissions.contains(Permissions::CREATE_EVENTS)
                || permissions.contains(Permissions::ADMINISTRATOR)
        }
        None => false,
    })
}
