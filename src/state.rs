use crate::giveaway::Giveaway;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

pub type State = Arc<Mutex<InnerState>>;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct InnerState {
    pub giveaways: Vec<Giveaway>,
}
