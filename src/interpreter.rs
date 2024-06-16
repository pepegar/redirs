use tokio::sync::RwLock;
use std::{collections::HashMap, sync::Arc};
use anyhow::{Result, anyhow};
use crate::commands::Command;

#[derive(Clone)]
pub struct Interpreter {
    cache: Arc<RwLock<HashMap<String, String>>>
}

impl Interpreter {
    pub async fn respond(self: &Self, cmd: Command) -> Result<Command> {
        match cmd {
            Command::PING => Ok(Command::PONG),
            Command::ECHO(x) => Ok(Command::ECHO(x)),
            Command::SET(key, value) => {
                let mut map = self.cache.write().await;
                map.insert(key, value);
                Ok(Command::OK)
            },
            Command::GET(key) => {
                let map = self.cache.read().await;
                match map.get(key.as_str()) {
                    Some(value) => Ok(Command::STR(value.to_owned())),
                    None => Ok(Command::NIL),
                }
            },
            x => Err(anyhow!("unexpected command: {:?}", x))
        }
    }

    pub(crate) fn new() -> Interpreter {
        Interpreter{
            cache: Arc::new(RwLock::new(HashMap::new()))
        }
    }
}
