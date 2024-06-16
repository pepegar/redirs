use log::info;
use rand::{thread_rng, RngCore};
use tokio::net::TcpStream;

use crate::{commands::CommandRequest, stream::CommandStream};


pub fn gen_replica_id() -> String {
    let mut rng = thread_rng();
    let mut bytes = vec![0u8; 40 / 2];
    rng.fill_bytes(&mut bytes);
    let hex_string: String = bytes.iter().map(|byte| format!("{:02x}", byte)).collect();
    hex_string
}

pub struct Replicator {
    master_address: String
}

impl Replicator {
    pub fn new(master_address: String) -> Replicator {
        Replicator {
            master_address
        }
    }

    pub async fn replicate(&self) {
        info!(target: "replicator", "replicating {:?}", self.master_address);
        let master_address = self.master_address.to_owned();
        let tcp_stream = TcpStream::connect(master_address).await.unwrap();
        let mut command_stream = CommandStream::from_tcp_stream(tcp_stream);
        command_stream.write_request(CommandRequest::PING).await.unwrap();
    }
}
