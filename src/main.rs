use protocol::{encode, Response};
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
                    let mut buf = [0; 512];
                    loop {
                        let read_count = stream.read(&mut buf).await.unwrap();
                        if read_count == 0 {
                            break;
                        }

                        stream.write(&encode(Response::PONG)).await.unwrap();
                    }
                });
            },
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
