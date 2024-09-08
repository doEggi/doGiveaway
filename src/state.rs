use crate::giveaway::Giveaway;
use std::sync::Arc;
use tokio::sync::Mutex;

pub type State = Arc<Mutex<InnerState>>;

#[derive(Debug, Default)]
pub struct InnerState {
    pub giveaways: Vec<Giveaway>,
}
