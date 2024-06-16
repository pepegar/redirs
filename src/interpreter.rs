use dashmap::DashMap;
use tokio::sync::{mpsc::Sender, Mutex, RwLock};
use std::{collections::HashMap, sync::Arc, time::Duration};
use anyhow::{Result, anyhow};
use crate::commands::Command;

#[derive(Clone)]
pub struct Interpreter {
    cache: Arc<DashMap<String, String>>,
    tx: Arc<Mutex<Sender<(String, Duration)>>>,
}

impl Interpreter {
    pub async fn respond(self: &Self, cmd: Command) -> Result<Command> {
        match cmd {
            Command::PING => Ok(Command::PONG),
            Command::ECHO(x) => Ok(Command::ECHO(x)),
            Command::SET(key, value, expiry) => {
                self.cache.insert(key.clone(), value);

                match expiry {
                    Some(millis) => {
                        let tx = self.tx.lock().await;
                        let _ = tx.send((key, Duration::from_millis(millis))).await;
                    },
                    None => (),
                }
                
                Ok(Command::OK)
            },
            Command::GET(key) => {
                match self.cache.get(key.as_str()) {
                    Some(value) => Ok(Command::STR(value.to_owned())),
                    None => Ok(Command::NIL),
                }
            },
            x => Err(anyhow!("unexpected command: {:?}", x))
        }
    }

    pub(crate) fn new(
        cache: Arc<DashMap<String, String>>,
        tx: Arc<Mutex<Sender<(String, Duration)>>>
    ) -> Interpreter {
        Interpreter{ cache, tx }
    }
}
