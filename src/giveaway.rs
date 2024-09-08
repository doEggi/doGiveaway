use anyhow::Result;
use chrono::{DateTime, Utc};
use poise::serenity_prelude::{ChannelId, CreateMessage, GuildId, Http, MessageId, UserId};
use rand::{seq::SliceRandom, thread_rng};
use std::{fmt::Display, num::NonZero};

#[derive(Debug)]
pub struct Giveaway {
    pub title: String,
    pub message: MessageId,
    pub channel: ChannelId,
    pub guild: GuildId,
    pub emoji: String,
    pub time: Option<DateTime<Utc>>,
    pub users: Vec<UserId>,
    pub winner_count: NonZero<u16>,
    pub id: GiveawayId,
}

impl Giveaway {
    pub async fn finish(self, http: impl AsRef<Http>) -> Result<()> {
        let mut message = format!("# {}\n\nGewinner: ", self.title);
        let winners: Vec<UserId> = self
            .users
            .choose_multiple(&mut thread_rng(), self.winner_count.get().into())
            .map(|val| *val)
            .collect();
        if winners.is_empty() {
            message.push_str("Niemand");
        } else {
            let mut i: u8 = 1;
            for winner in &winners {
                message.push_str(&format!("\n{}. <@{}>", i, winner.get()));
                i += 1;
            }
        }
        message.push_str(&format!(
            "\n\nÖffne ein Ticket um deinen Preis zu erhalten!\n||{}||",
            self.id.0.get()
        ));
        _ = self.channel.delete_message(&http, self.message).await;
        _ = self
            .channel
            .send_message(http.as_ref(), CreateMessage::new().content(message))
            .await?;
        println!(
            "Finished giveaway: {}\n{:?}\n{:?}",
            self.title, self.users, winners
        );
        Ok(())
    }

    pub async fn cancel(self, http: impl AsRef<Http>, user: &UserId) -> Result<()> {
        let message = format!(
            "# {}\n\nDieses Giveaway wurde von <@{}> abgebrochen.\n||{}||",
            self.title,
            user.get(),
            self.id.0.get()
        );
        _ = self.channel.delete_message(&http, self.message).await;
        _ = self
            .channel
            .send_message(http.as_ref(), CreateMessage::new().content(message))
            .await?;
        println!(
            "Canceled giveaway: {}\n{:?}\n{:?}",
            self.title, self.users, user
        );
        Ok(())
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct GiveawayId(pub NonZero<u32>);

impl GiveawayId {
    pub fn new() -> Self {
        //  HOPEFULLY there will never be a collision (which is unlikely)
        //  I fixed the collision problem while creating the ids in commands.rs > create()
        GiveawayId(rand::random())
    }
}

impl Display for GiveawayId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("{}", self.0))
    }
}
