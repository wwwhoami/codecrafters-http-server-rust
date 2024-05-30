pub mod request;
pub mod response;

use anyhow::Result;
use request::{HTTPError, Request};
use response::ResponseBuilder;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};

#[tokio::main]
async fn main() -> Result<()> {
    const HOST: &str = "127.0.0.1";
    const PORT: i32 = 4221;
    let addr = format!("{}:{}", HOST, PORT);

    let listener = TcpListener::bind(addr).await?;

    println!("server listening on port {}", PORT);

    loop {
        let (stream, _) = listener.accept().await?;

        tokio::spawn(async move {
            let _ = handle_stream(stream).await;
        });
    }
}

async fn handle_stream(mut stream: TcpStream) -> Result<()> {
    let request = read_request(&mut stream).await;

    let response = match request {
        Ok(request) => {
            println!("{}", request);

            match request.request_line().path() {
                "/" => ResponseBuilder::new().status(200, "OK").build(),
                "/user-agent" => {
                    let default_agent = "Unknown".to_string();
                    let user_agent = request
                        .headers()
                        .get("User-Agent")
                        .unwrap_or(&default_agent);

                    ResponseBuilder::new()
                        .status(200, "OK")
                        .header("Content-Type", "text/plain")
                        .body(user_agent.as_bytes())
                        .build()
                }
                _ => {
                    if request.request_line().path().starts_with("/echo") {
                        let echo_string = request.request_line().path().replace("/echo/", "");
                        let echo_string = echo_string.replace("%20", " ");

                        ResponseBuilder::new()
                            .status(200, "OK")
                            .header("Content-Type", "text/plain")
                            .body(echo_string.as_bytes())
                            .build()
                    } else {
                        ResponseBuilder::new().status(404, "Not Found").build()
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("HTTP Error: {:?}", e);
            ResponseBuilder::new().status(400, "Bad Request").build()
        }
    };

    let response = response?;

    println!("Response: {:?}", response);

    stream.write_all(&response.as_bytes()).await?;

    Ok(())
}

async fn read_request(stream: &mut TcpStream) -> Result<Request, HTTPError> {
    let buffer = read_from_stream(stream).await?;

    let str_buffer = String::from_utf8(buffer.to_vec())?;

    Request::parse_request(&str_buffer)
}

async fn read_from_stream(stream: &mut TcpStream) -> Result<Vec<u8>, std::io::Error> {
    let mut buf_reader = tokio::io::BufReader::new(stream);
    let mut buffer = Vec::new();

    loop {
        let mut buf = [0; 1024];
        let bytes_read = buf_reader.read(&mut buf[..]).await?;

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
