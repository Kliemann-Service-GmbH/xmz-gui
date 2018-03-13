use std::sync::{Arc, Mutex};
use std::sync::mpsc::Sender;

use error::BackendError;

// use types::Server;
// use types::Messzelle;

#[derive(Debug)]
pub enum BKCommand {
    ShutDown,
}

#[derive(Debug)]
pub enum BKResponse {
    ShutDown,
}

pub struct BackendData {

}

pub struct Backend {
    pub tx: Sender<BKResponse>,
    pub data: Arc<Mutex<BackendData>>,
    pub internal_tx: Option<Sender<BKCommand>>,
}

impl Clone for Backend {
    fn clone(&self) -> Backend {
        Backend {
            tx: self.tx.clone(),
            data: self.data.clone(),
            internal_tx: self.internal_tx.clone(),
        }
    }
}
