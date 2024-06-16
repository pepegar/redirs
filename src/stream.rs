use std::str::from_utf8;

use anyhow::{anyhow, Result};
use log::debug;
use tokio::{io::{AsyncReadExt, AsyncWriteExt}, net::TcpStream};

use crate::{commands::{CommandRequest, CommandResponse, FromRESP, ToRESP}, protocol::RESP};

pub struct RESPStream {
    tcp_stream: TcpStream
}

impl RESPStream {
    pub async fn write(&mut self, resp: RESP) -> Result<usize> {
        self.tcp_stream.write(
            resp.encode().as_bytes()
        ).await.map_err(|err| anyhow!("got io error: {err:?}"))
    }

    pub async fn receive(&mut self) -> Result<RESP> {
        let mut buffer = vec![0; 512];
        let n = self.tcp_stream.read(&mut buffer).await?;
        let str = from_utf8(&buffer[..n]).unwrap();
        debug!(target: "resp-stream", "receiving bytes: {str:?}");
        RESP::decode(str)
    }

    pub fn new(tcp_stream: TcpStream) -> RESPStream {
        RESPStream {
            tcp_stream
        }
    }    
}

pub struct CommandStream {
    resp_stream: RESPStream
}

impl CommandStream {
    pub fn from_tcp_stream(tcp_stream: TcpStream) -> CommandStream {
        CommandStream {
            resp_stream: RESPStream::new(tcp_stream)
        }
    }
    
    pub fn new(resp_stream: RESPStream) -> CommandStream {
        CommandStream {
            resp_stream
        }
    }
    
    pub async fn write_response(&mut self, command: CommandResponse) -> Result<usize> {
        debug!(target: "command-stream", "writing command response: {command:?}");
        self.resp_stream.write(
            command.to_resp().unwrap()
        ).await
    }
    
    pub async fn write_request(&mut self, command: CommandRequest) -> Result<usize> {
        debug!(target: "command-stream", "writing command request: {command:?}");
        self.resp_stream.write(
            command.to_resp().unwrap()
        ).await
    }

    pub async fn receive_request(&mut self) -> Result<CommandRequest> {
        let resp = self.resp_stream.receive().await.unwrap();
        debug!(target: "command-stream", "receiving RESP: {resp:?}");
        CommandRequest::from_resp(resp)
    }    
}
