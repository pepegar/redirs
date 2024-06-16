use anyhow::{Result, anyhow};

use crate::protocol::RESP;

pub trait FromRESP {
    fn from_resp(resp: RESP) -> Result<Command>;
}

pub trait ToRESP {
    fn to_resp(self: &Self) -> Result<RESP>;
}

#[derive(Debug)]
pub enum Command {
    PING,
    PONG,
    ECHO(String)
}

impl FromRESP for Command {
    fn from_resp(resp: RESP) -> Result<Command> {
        match resp {
            RESP::Array(commands) => {
                match commands.as_slice() {
                    [RESP::BulkString("ECHO"), RESP::BulkString(x)] => {
                        Ok(Command::ECHO(x.to_string()))
                    },
                    [RESP::BulkString("PING") | RESP::SimpleString("PING")] => {
                        Ok(Command::PING)
                    },
                    _ => todo!(),
                }
            },
            x => Err(anyhow!("unexpected RESP command: {:?}", x))
        }
    }
}

impl ToRESP for Command {
    fn to_resp(self: &Self) -> Result<RESP> {
        match self {
            Command::PONG => Ok(RESP::BulkString("PONG")),
            Command::ECHO(x) => Ok(RESP::BulkString(x)),
            x => Err(anyhow!("unexpected command in ToRESP: {:?}", x))
        }
    }
}
