use names::Generator;
use std::net::SocketAddr;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::TcpListener,
    sync::broadcast,
};

#[derive(Clone)]
struct User {
    name: String,
    addr: SocketAddr,
}

impl User {
    pub fn new(addr: SocketAddr) -> User {
        let mut generator = Generator::default();

        User {
            name: generator.next().unwrap(),
            addr: addr,
        }
    }
}

struct Room {
    users: Vec<User>,
}

impl Room {
    pub fn new() -> Room {
        Room { users: Vec::new() }
    }
}

#[tokio::main]
async fn main() {
    println!("Everything is not debugged...");

    let mut room = Room::new();

    let listener: TcpListener = TcpListener::bind("0.0.0.0:7228").await.unwrap();

    let (tx, _rx) = broadcast::channel(10);

    loop {
        let (mut socket, addr) = listener.accept().await.unwrap();

        let user = User::new(addr);
        room.users.push(user.clone());

        let tx = tx.clone();
        let mut rx = tx.subscribe();

        tokio::spawn(async move {
            let (reader, mut writer) = socket.split();

            let mut reader = BufReader::new(reader);
            let mut line = String::new();

            loop {
                tokio::select! {
                    result = reader.read_line(&mut line) => {
                        if result.unwrap() == 0 {
                            break;
                        }

                        tx.send(
                            (
                                format!("[{}] {}", user.name, line.clone()),
                                user.addr
                            )
                        )
                        .unwrap();
                        line.clear();
                    }

                    result = rx.recv() => {
                        let (msg, other_addr) = result.unwrap();

                        if other_addr != user.addr {
                            writer.write_all(msg.as_bytes()).await.unwrap();
                        }
                    }
                }
            }
        });
    }
}
