pub mod request;

use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
};

use request::{HTTPError, Request};

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

                handle_stream(stream);
            }
            Err(e) => {
                eprintln!("error: {}", e);
            }
        }
    }
}

fn handle_stream(mut stream: TcpStream) {
    let request = read_request(&mut stream);
    println!("Request: {:?}", request);

    match request {
        Ok(request) => match request.request_line().path() {
            "/" => {
                let response = format!("HTTP/1.1 200 OK\r\nContent-Length: 0\r\n\r\n");

                stream.write(response.as_bytes()).unwrap();
            }
            _ => {
                let response = format!("HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\n\r\n");

                stream.write(response.as_bytes()).unwrap();
            }
        },
        Err(e) => {
            eprintln!("error: {:?}", e);
        }
    }
}

fn read_request(stream: &mut TcpStream) -> Result<Request, HTTPError> {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).unwrap();

    // Remove trailing zeros from the buffer
    let buffer = buffer
        .iter()
        .map(|b| *b)
        .take_while(|b| *b != 0)
        .collect::<Vec<u8>>();

    let buffer = String::from_utf8(buffer.to_vec()).unwrap();

    Request::parse_request(&buffer)
}
