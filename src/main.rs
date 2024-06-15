use std::io::Write;
use std::net::TcpListener;

use protocol::encode;
use protocol::Response;

fn main() {
    println!("Logs from your program will appear here!");

    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                let mut encoded = encode(Response::PONG);
                encoded.extend(encode(Response::PONG));
                stream.write(&encoded).unwrap();
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

mod protocol {
    pub enum Response {
        PONG
    }

    impl Response {
        fn as_str(&self) -> &str {
            match self {
                Self::PONG => "PONG",
            }
        }
    }

    pub fn encode(cmd: Response) -> Vec<u8> {
        return format!("+{}\r\n", cmd.as_str()).as_bytes().to_vec();
    }
}
