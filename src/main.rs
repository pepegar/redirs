mod protocol;
mod interpreter;
mod commands;

use std::str::from_utf8;

use commands::{Command, FromRESP, ToRESP};
use interpreter::Interpreter;
use protocol::RESP;
use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};


#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();

    loop {
        let stream = listener.accept().await;
        match stream {
            Ok((mut stream, _)) => {
                tokio::spawn(async move {
                    let interpreter = Interpreter::new();
                    let mut buf = [0; 512];
                    loop {
                        let read_count = stream.read(&mut buf).await.unwrap();
                        if read_count == 0 {
                            break;
                        }

                        let str = from_utf8(&buf).unwrap();

                        let resp = RESP::decode(str).unwrap();
                        let command = Command::from_resp(resp).unwrap();
                        
                        stream.write(
                            /// now, this is ugly
                            interpreter.respond(command).unwrap().to_resp().unwrap().encode().as_bytes()
                        ).await.unwrap();
                    }
                });
            },
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
