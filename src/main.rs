use std::{io::Write, net::TcpListener};

fn main() {
    const HOST: &str = "127.0.0.1";
    const PORT: i32 = 4221;
    let addr = format!("{}:{}", HOST, PORT);

    let listener = TcpListener::bind(addr).unwrap();

    println!("server listening on port {}", PORT);

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                println!(
                    "accepted new connection from: {}",
                    stream.peer_addr().unwrap()
                );
                let res = b"HTTP/1.1 200 OK\r\n";
                let write_result = stream.write(res);

                match write_result {
                    Ok(_) => {
                        println!("response sent");
                    }
                    Err(e) => {
                        println!("error sending response: {}", e);
                    }
                }
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
