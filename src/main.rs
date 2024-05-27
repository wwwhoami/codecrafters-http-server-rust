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
    // println!("Request: {:?}", request);

    let response = match request {
        Ok(request) => {
            println!("{}", request);

            match request.request_line().path() {
                "/" => "HTTP/1.1 200 OK\r\nContent-Length: 0\r\n\r\n".to_string(),
                _ => "HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\n\r\n".to_string(),
            }
        }
        Err(e) => {
            eprintln!("Error: {:?}", e);
            "HTTP/1.1 400 Bad Request\r\nContent-Length: 0\r\n\r\n".to_string()
        }
    };

    println!("Response: {:?}", response);

    stream.write_all(response.as_bytes()).unwrap();
}

fn read_request(stream: &mut TcpStream) -> Result<Request, HTTPError> {
    let buffer = read_from_stream(stream)?;

    let str_buffer = String::from_utf8(buffer.to_vec())?;

    Request::parse_request(&str_buffer)
}

fn read_from_stream(stream: &mut TcpStream) -> Result<Vec<u8>, std::io::Error> {
    let mut buf_reader = std::io::BufReader::new(stream);
    let mut buffer = Vec::new();

    loop {
        let mut buf = [0; 1024];
        let bytes_read = buf_reader.read(&mut buf)?;

        // If we read 0 bytes, we've reached the end of the stream
        if bytes_read == 0 {
            break;
        }

        buffer.extend_from_slice(&buf[..bytes_read]);

        // If we read less than the buffer size, we've read the entire request
        if bytes_read < buf.len() {
            break;
        }
    }

    Ok(buffer)
}
