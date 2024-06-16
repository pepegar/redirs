use anyhow::{Result, anyhow};

use crate::protocol::RESP;

pub trait FromRESP {
    fn from_resp(resp: RESP) -> Result<CommandRequest>;
}

pub trait ToRESP {
    fn to_resp(self: &Self) -> Result<RESP>;
}

#[derive(Debug)]
pub enum CommandRequest {
    PING,
    ECHO(String),
    GET(String),
    SET(String, String, Option<u64>),
    STR(String),
    DOCS,
}

#[derive(Debug)]
pub enum CommandResponse {
    PONG,
    ECHO(String),
    OK,
    STR(String),
    DOCS,
    NIL,
}

impl FromRESP for CommandRequest {
    fn from_resp(resp: RESP) -> Result<CommandRequest> {
        match resp {
            RESP::Array(commands) => {
                match commands.as_slice() {
                    [RESP::BulkString("COMMAND"), RESP::BulkString("DOCS")] => {
                        Ok(CommandRequest::DOCS)
                    },
                    [RESP::BulkString("ECHO"), RESP::BulkString(x)] => {
                        Ok(CommandRequest::ECHO(x.to_string()))
                    },
                    [RESP::BulkString("PING") | RESP::SimpleString("PING")] => {
                        Ok(CommandRequest::PING)
                    },
                    [RESP::BulkString("SET"), RESP::BulkString(key), RESP::BulkString(value)] => {
                        Ok(CommandRequest::SET(key.to_string(), value.to_string(), None))
                    },
                    [RESP::BulkString("GET"), RESP::BulkString(key)] => {
                        Ok(CommandRequest::GET(key.to_string()))
                    },
                    [RESP::BulkString("SET"),
                     RESP::BulkString(key),
                     RESP::BulkString(value),
                     RESP::BulkString("px"),
                     RESP::BulkString(expiry)] => {
                         let expiry_long = expiry.parse::<u64>().unwrap();
                         Ok(CommandRequest::SET(key.to_string(), value.to_string(), Some(expiry_long)))
                     },
                    x => Err(anyhow!("unexpected RESP command: {:?}", x)),
                }
            },
            x => Err(anyhow!("unexpected RESP command: {:?}", x))
        }
    }
}

impl ToRESP for CommandResponse {
    fn to_resp(self: &Self) -> Result<RESP> {
        match self {
            CommandResponse::PONG => Ok(RESP::BulkString("PONG")),
            CommandResponse::ECHO(x) => Ok(RESP::BulkString(x)),
            CommandResponse::OK => Ok(RESP::SimpleString("OK")),
            CommandResponse::STR(str) => Ok(RESP::BulkString(str)),
            CommandResponse::DOCS => Ok(RESP::BulkString("welcome to redis")),
            CommandResponse::NIL => Ok(RESP::NullBulkString),
            x => Err(anyhow!("unexpected command in ToRESP: {:?}", x))
        }
    }
}
