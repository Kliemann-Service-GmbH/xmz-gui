

use std::sync::{Arc, Mutex};
use std::thread;
use std::sync::mpsc::{channel, Sender, Receiver, RecvError};

use error::Error;

mod types;

pub use self::types::{BKResponse, BKCommand, Backend, BackendData};

impl Backend {
    pub fn new(tx: Sender<BKResponse>) -> Backend {
        let data = BackendData { };
        Backend {
            tx: tx,
            internal_tx: None,
            data: Arc::new(Mutex::new(data)),
        }
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
