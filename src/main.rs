mod protocol;
mod interpreter;
mod commands;
mod expirator;
mod replication;
mod stream;

use commands::{CommandRequest, FromRESP, ToRESP};
use interpreter::Interpreter;
use protocol::RESP;
use expirator::Expirator;
use replication::{gen_replica_id, Replicator};
use stream::CommandStream;

use dashmap::DashMap;
use log::info;
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
    let mut replica_of = None;
    let replica_id = gen_replica_id();
    info!("{args:?}");

    match args.as_slice() {
        [_, flag, p] if flag == "--port" => port = p,
        [_, r_flag, r, p_flag, p] if p_flag == "--port" && r_flag == "--replicaof" => {
            port = p;
            replication_role = ReplicationRole::Slave;
            let parts: Vec<&str> = r.split(' ').collect();
            replica_of = Some(format!("{}:{}", parts[0], parts[1]))
        },
        [_, p_flag, p, r_flag, r] if p_flag == "--port" && r_flag == "--replicaof" => {
            port = p;
            replication_role = ReplicationRole::Slave;
            let parts: Vec<&str> = r.split(' ').collect();
            replica_of = Some(format!("{}:{}", parts[0], parts[1]))
        },
        _ => ()
    }

    let (tx, rx) = mpsc::channel::<(String, Duration)>(32);
    let address = "127.0.0.1:".to_string() + port;
    let listener = TcpListener::bind(address).await.unwrap();
    info!(target: "main", "running as {replication_role:?}, with replica_id: {replica_id:?}, listening on port {port:?}");
    let cache = Arc::new(DashMap::new());
    let tx_protected = Arc::new(Mutex::new(tx));
    let interpreter = Interpreter::new(replica_id, replication_role, cache.clone(), tx_protected);
    let rx_protected = Arc::new(Mutex::new(rx));
    let expirator = Expirator::new(rx_protected.clone(), cache.clone());
    let expirator_clone = expirator.clone();

    tokio::spawn(async move {
        info!(target: "main", "running expirator");
        expirator_clone.listen().await;
    });

    match replica_of {
        Some(master_address) => {
            let replicator = Replicator::new(master_address);
            tokio::spawn(async move {
                replicator.replicate().await
            });
        },
        None => (),
    }
    
    loop {
        let stream = listener.accept().await.unwrap().0;
        info!(target: "main", "receiving request");
        let mut server_stream = CommandStream::from_tcp_stream(stream);
        let interp_clone = interpreter.clone();
        tokio::spawn(async move {
            loop {
                let command = server_stream.receive_request().await.unwrap();
                info!(target: "main", "parsed as command: {command:?}");
                let command_response = interp_clone.respond(command).await.unwrap();
                info!(target: "main", "answering with : {command_response:?}");
                let _ = server_stream.write_response(command_response).await.unwrap();
            }
        });
    }
}
