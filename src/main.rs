use std::net::TcpListener;

fn main() {
    const HOST: &str = "127.0.0.1";
    const PORT: i32 = 4221;
    let addr = format!("{}:{}", HOST, PORT);

    let listener = TcpListener::bind(addr).unwrap();

    println!("server listening on port {}", PORT);

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!(
                    "accepted new connection from: {}",
                    stream.peer_addr().unwrap()
                );
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
