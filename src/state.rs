use crate::giveaway::Giveaway;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use songbird::typemap::TypeMapKey;
use std::sync::Arc;
use tokio::sync::Mutex;

pub type State = Arc<Mutex<InnerState>>;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct InnerState {
    pub giveaways: Vec<Giveaway>,
}

pub(crate) struct HttpKey;

impl TypeMapKey for HttpKey {
    type Value = Client;
}
