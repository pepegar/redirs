mod protocol;
mod interpreter;
mod commands;
mod expirator;

use commands::{Command, FromRESP, ToRESP};
use dashmap::DashMap;
use interpreter::Interpreter;
use log::info;
use protocol::RESP;
use expirator::Expirator;
use env_logger;

use std::str::from_utf8;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, Mutex, RwLock};
use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};


#[tokio::main]
async fn main() {
    env_logger::init();
    
    let (tx, rx) = mpsc::channel::<(String, Duration)>(32);
    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();
    let cache = Arc::new(DashMap::new());
    let tx_protected = Arc::new(Mutex::new(tx));
    let interpreter = Interpreter::new(cache.clone(), tx_protected);
    let rx_protected = Arc::new(Mutex::new(rx));
    let expirator = Expirator::new(rx_protected.clone(), cache.clone());
    let expirator_clone = expirator.clone();

    tokio::spawn(async move {
        info!(target: "main", "running expirator");
        expirator_clone.listen().await;
    });
    
    loop {
        let stream = listener.accept().await;
        match stream {
            Ok((mut stream, _)) => {
                info!(target: "main", "receiving request");
                let interp_clone = interpreter.clone();
                tokio::spawn(async move {
                    let mut buf = [0; 512];
                    loop {
                        let read_count = stream.read(&mut buf).await.unwrap();
                        if read_count == 0 {
                            break;
                        }

                        let str = from_utf8(&buf).unwrap();

                        let resp = RESP::decode(str).unwrap();
                        let command = Command::from_resp(resp).unwrap();

                        info!(target: "main", "parsed as command: {command:?}");
                        
                        stream.write(
                            interp_clone
                                .respond(command)
                                .await
                                .unwrap()
                                .to_resp()
                                .unwrap()
                                .encode()
                                .as_bytes()
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
