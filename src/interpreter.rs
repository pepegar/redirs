use dashmap::DashMap;
use tokio::sync::{mpsc::Sender, Mutex};
use std::{sync::Arc, time::Duration};
use anyhow::{Result, anyhow};
use crate::commands::{CommandRequest, CommandResponse, InfoMode, ReplicationInfo, ReplicationRole};

#[derive(Clone)]
pub struct Interpreter {
    role: ReplicationRole,
    replica_id: String,
    master_repl_offset: u8,
    cache: Arc<DashMap<String, String>>,
    tx: Arc<Mutex<Sender<(String, Duration)>>>,
}

impl Interpreter {
    pub async fn respond(self: &Self, cmd: CommandRequest) -> Result<CommandResponse> {
        match cmd {
            CommandRequest::PING => Ok(CommandResponse::PONG),
            CommandRequest::ECHO(x) => Ok(CommandResponse::ECHO(x)),
            CommandRequest::SET(key, value, expiry) => {
                self.cache.insert(key.clone(), value);

                match expiry {
                    Some(millis) => {
                        let tx = self.tx.lock().await;
                        let _ = tx.send((key, Duration::from_millis(millis))).await;
                    },
                    None => (),
                }
                
                Ok(CommandResponse::OK)
            },
            CommandRequest::GET(key) => {
                match self.cache.get(key.as_str()) {
                    Some(value) => Ok(CommandResponse::STR(value.to_owned())),
                    None => Ok(CommandResponse::NIL),
                }
            },
            CommandRequest::DOCS => Ok(CommandResponse::DOCS),
            CommandRequest::INFO(InfoMode::Replication) => Ok(CommandResponse::INFO(
                ReplicationInfo::new(
                    self.role.clone(),
                    self.replica_id.to_owned(),
                    self.master_repl_offset
                )
            )),
            x => Err(anyhow!("unexpected command: {:?}", x))
        }
    }

    pub(crate) fn new(
        replica_id: String,
        role: ReplicationRole,
        cache: Arc<DashMap<String, String>>,
        tx: Arc<Mutex<Sender<(String, Duration)>>>
    ) -> Interpreter {
        Interpreter{
            replica_id,
            master_repl_offset: 0,
            role,
            cache,
            tx
        }
    }
}
