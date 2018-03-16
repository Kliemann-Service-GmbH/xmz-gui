use std::sync::{Arc, Mutex};
use std::sync::mpsc::Sender;

use error::Error;

use types::Server;
// use types::Messzelle;

#[derive(Debug)]
pub enum BKCommand {
    // Verbinde zu einem Server
    Connect(String),
    ShutDown,
    Sync,
}

#[derive(Debug)]
pub enum BKResponse {
    ConnectSuccessfull,
    ShutDown,
    Sync(String),
    SyncError(Error),
    LoginError(Error),
}

pub struct BackendData {
    pub server_url: String,
    pub since: String,
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
