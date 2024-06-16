mod protocol;
mod interpreter;
mod commands;
mod expirator;

use commands::{CommandRequest, FromRESP, ToRESP};
use dashmap::DashMap;
use interpreter::Interpreter;
use log::info;
use protocol::RESP;
use expirator::Expirator;
use env_logger;

use std::env;
use std::str::from_utf8;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, Mutex};
use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::commands::ReplicationRole;


#[tokio::main]
async fn main() {
    env_logger::init();
    let args: Vec<String> = env::args().collect();

    let mut port = "6379";
    let mut replication_role = ReplicationRole::Master;
    info!("{args:?}");

    match args.as_slice() {
        [_, flag, p] if flag == "--port" => port = p,
        [_, r_flag, r, p_flag, p] if p_flag == "--port" && r_flag == "--replicaof" => {
            port = p;
            replication_role = ReplicationRole::Slave;
        },
        [_, p_flag, p, r_flag, r] if p_flag == "--port" && r_flag == "--replicaof" => {
            port = p;
            replication_role = ReplicationRole::Slave;
        },
        _ => ()
    }

    let (tx, rx) = mpsc::channel::<(String, Duration)>(32);
    let address = "127.0.0.1:".to_string() + port;
    let listener = TcpListener::bind(address).await.unwrap();
    info!(target: "main", "running a {replication_role:?}, listening on port {port:?}");
    let cache = Arc::new(DashMap::new());
    let tx_protected = Arc::new(Mutex::new(tx));
    let interpreter = Interpreter::new(replication_role, cache.clone(), tx_protected);
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
                        let command = CommandRequest::from_resp(resp).unwrap();

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
