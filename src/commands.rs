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
    ECHO(String),
    GET(String),
    SET(String, String, Option<usize>),
    OK,
    STR(String),
    NIL,
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
                    [RESP::BulkString("SET"), RESP::BulkString(key), RESP::BulkString(value)] => {
                        Ok(Command::SET(key.to_string(), value.to_string(), None))
                    },
                    [RESP::BulkString("GET"), RESP::BulkString(key)] => {
                        Ok(Command::GET(key.to_string()))
                    },
                    [RESP::BulkString("SET"),
                     RESP::BulkString(key),
                     RESP::BulkString(value),
                     RESP::BulkString("px"),
                     RESP::BulkString(expiry)] => {
                         let expiry_long = expiry.parse::<usize>().unwrap();
                         Ok(Command::SET(key.to_string(), value.to_string(), Some(expiry_long)))
                     },
                    x => Err(anyhow!("unexpected RESP command: {:?}", x)),
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
            Command::OK => Ok(RESP::SimpleString("OK")),
            Command::STR(str) => Ok(RESP::BulkString(str)),
            Command::NIL => Ok(RESP::Null),
            x => Err(anyhow!("unexpected command in ToRESP: {:?}", x))
        }
    }
}
