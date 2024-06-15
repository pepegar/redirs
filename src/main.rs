use std::net::TcpListener;

use protocol::encode;
use protocol::Response;

fn main() {
    println!("Logs from your program will appear here!");

    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(_stream) => {
                println!("{}", encode(Response::PONG));
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

    pub fn encode(cmd: Response) -> String {
        return format!("+{}\r\n", cmd.as_str());
    }
}
