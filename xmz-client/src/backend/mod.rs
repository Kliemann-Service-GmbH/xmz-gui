mod sync;
mod types;
mod register;

pub use self::types::{BKResponse, BKCommand, Backend, BackendData};

use error::Error;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{channel, Sender, Receiver, RecvError};
use std::thread;
use url::Url;
use utils::build_url;


impl Backend {
    pub fn new(tx: Sender<BKResponse>) -> Backend {
        let data = BackendData {
            server_url: "http://0.0.0.0:3000".to_string(),
            since: "".to_string(),
        };
        Backend {
            tx: tx,
            internal_tx: None,
            data: Arc::new(Mutex::new(data)),
        }
    }

    fn get_base_url(&self) -> Result<Url, Error> {
        let s = self.data.lock().unwrap().server_url.clone();
        let url = Url::parse(&s)?;
        Ok(url)
    }

    fn url(&self, path: &str, params: Vec<(&str, String)>) -> Result<Url, Error> {
        let base = self.get_base_url()?;

        client_url!(&base, path, params)
    }

    pub fn run(mut self) -> Sender<BKCommand> {
        let (apptx, rx): (Sender<BKCommand>, Receiver<BKCommand>) = channel();

        self.internal_tx = Some(apptx.clone());
        thread::spawn(move || loop {
            let cmd = rx.recv();
            if !self.command_recv(cmd) {
                break;
            }
        });

        apptx
    }

    pub fn command_recv(&mut self, cmd: Result<BKCommand, RecvError>) -> bool {
        let tx = self.tx.clone();

        match cmd {
            // Server Commandos
            Ok(BKCommand::Connect(url)) => {
                let r = register::login(self, url.clone());
                if let Err(e) = r {
                    tx.send(BKResponse::LoginError(e));
                }
            }

            // Sync Modul
            Ok(BKCommand::Sync) => {
                let r = sync::sync(self);
                if let Err(e) = r {
                    tx.send(BKResponse::SyncError(e));
                }
            }

            // internal commands
            Ok(BKCommand::ShutDown) => {
                tx.send(BKResponse::ShutDown).unwrap();
                return false;
            },

            Err(_) => {
                return false;
            }
        };

        true
    }
}
