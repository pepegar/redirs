use anyhow::{Result, anyhow};

use crate::protocol::RESP;

pub trait FromRESP {
    fn from_resp(resp: RESP) -> Result<CommandRequest>;
}

pub trait ToRESP {
    fn to_resp(self: &Self) -> Result<RESP>;
}

#[derive(Debug)]
pub enum InfoMode {
    Replication
}

#[derive(Debug)]
pub enum CommandRequest {
    PING,
    ECHO(String),
    GET(String),
    SET(String, String, Option<u64>),
    DOCS,
    INFO(InfoMode)
}

#[derive(Debug)]
pub enum ReplicationRole {
    Master,
    Slave
}

impl ReplicationRole {
    pub fn to_str(&self) -> &str {
        match self {
            ReplicationRole::Master => "master",
            ReplicationRole::Slave => "slave",
        }
    }

}

#[derive(Debug)]
pub struct ReplicationInfo {
    role: ReplicationRole,
    connected_slaves: u8,
    master_replid: String,
    master_repl_offset: u8,
    second_repl_offset: i8,
    repl_backlog_active: u8,
    repl_backlog_size: u64,
    repl_backlog_first_byte_offset: u64,
    repl_backlog_histlen: u64
}
impl ReplicationInfo {
    pub(crate) fn new() -> ReplicationInfo {
        ReplicationInfo {
            role: ReplicationRole::Master,
            connected_slaves: 0,
            master_replid: "test".to_string(),
            master_repl_offset: 0,
            second_repl_offset: 0,
            repl_backlog_active: 0,
            repl_backlog_size: 0,
            repl_backlog_first_byte_offset: 0,
            repl_backlog_histlen: 0
        }
    }
}

#[derive(Debug)]
pub enum CommandResponse {
    PONG,
    ECHO(String),
    OK,
    STR(String),
    INFO(ReplicationInfo),
    DOCS,
    NIL,
}

impl FromRESP for CommandRequest {
    fn from_resp(resp: RESP) -> Result<CommandRequest> {
        match resp {
            RESP::Array(commands) => {
                match commands.as_slice() {
                    [RESP::BulkString(c), RESP::BulkString(d)] if *c == "COMMAND" && *d == "DOCS" => {
                        Ok(CommandRequest::DOCS)
                    },
                    [RESP::BulkString(e), RESP::BulkString(x)] if *e == "ECHO" => {
                        Ok(CommandRequest::ECHO(x.to_string()))
                    },
                    [RESP::BulkString(x) | RESP::SimpleString(x)] if *x == "PING" => {
                        Ok(CommandRequest::PING)
                    },
                    [RESP::BulkString(s), RESP::BulkString(key), RESP::BulkString(value)] if *s == "SET" => {
                        Ok(CommandRequest::SET(key.to_string(), value.to_string(), None))
                    },
                    [RESP::BulkString(g), RESP::BulkString(key)] if *g == "GET" => {
                        Ok(CommandRequest::GET(key.to_string()))
                    },
                    [RESP::BulkString(s),
                     RESP::BulkString(key),
                     RESP::BulkString(value),
                     RESP::BulkString(px),
                     RESP::BulkString(expiry)] if *s == "SET" && *px == "px" => {
                         let expiry_long = expiry.parse::<u64>().unwrap();
                         Ok(CommandRequest::SET(key.to_string(), value.to_string(), Some(expiry_long)))
                     },
                    [RESP::BulkString(i), RESP::BulkString(r)] if *i == "INFO" && r == "replication" => Ok(CommandRequest::INFO(InfoMode::Replication)),
                    x => Err(anyhow!("unexpected RESP command: {:?}", x)),
                }
            },
            x => Err(anyhow!("unexpected RESP command: {:?}", x))
        }
    }
}

impl ToRESP for CommandResponse {
    fn to_resp(&self) -> Result<RESP> {
        match self {
            CommandResponse::PONG => Ok(RESP::BulkString("PONG".to_string())),
            CommandResponse::ECHO(x) => Ok(RESP::BulkString(x.to_string())),
            CommandResponse::OK => Ok(RESP::SimpleString("OK".to_string())),
            CommandResponse::STR(str) => Ok(RESP::BulkString(str.to_string())),
            CommandResponse::DOCS => Ok(RESP::BulkString("welcome to redis".to_string())),
            CommandResponse::NIL => Ok(RESP::NullBulkString),
            CommandResponse::INFO(r) => Ok(
                RESP::BulkString(
                    [
                        ["role", r.role.to_str()].join(":"),
                        ["connected_slaves", "0"].join(":"),
                        ["master_replid", "8371b4fb1155b71f4a04d3e1bc3e18c4a990aeeb"].join(":"),
                        ["master_repl_offset", "0"].join(":"),
                        ["second_repl_offset:", "1"].join(":"),
                        ["repl_backlog_active", "0"].join(":"),
                        ["repl_backlog_size", "1048576"].join(":"),
                        ["repl_backlog_first_byte_offset", "0"].join(":"),
                        ["repl_backlog", "histlen:"].join(":")
                    ].join("\r\n")
                )
            ),
            x => Err(anyhow!("unexpected command in ToRESP: {:?}", x))
        }
    }
}
