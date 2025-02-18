pub mod config;
pub mod middleware;
pub mod request;
pub mod response;
pub mod server;
pub mod utils;

use std::{env, fs};

use anyhow::Result;
use config::Config;

use response::ResponseBuilder;
use server::{RequestInfo, Server};

#[tokio::main]
async fn main() -> Result<()> {
    let config = Config::new(env::args()).unwrap_or_else(|err| {
        eprintln!("Problem parsing arguments: {}", err);
        std::process::exit(1);
    });
    let addr = format!("127.0.0.1:{}", config.port);
    let socket_addr = std::net::SocketAddr::V4(addr.parse().unwrap());

    let mut server = Server::new(socket_addr, config).await?;

    server.route_handlers(&[
        ("GET /", |_| {
            let response = ResponseBuilder::new().status(200, "OK");
            Ok(response)
        }),
        ("GET /user-agent", |req_info| {
            let request = req_info.request();

            let default_agent = "Unknown".to_string();

            let user_agent = request
                .headers()
                .get("User-Agent")
                .unwrap_or(&default_agent);

            let response = ResponseBuilder::new()
                .status(200, "OK")
                .header("Content-Type", "text/plain")
                .body(user_agent.as_bytes());

            Ok(response)
        }),
        ("GET /echo/:whatToEcho", |req_info| {
            let request = req_info.request();

            let echo_string = request.params().get("whatToEcho").unwrap();
            let echo_string = echo_string.replace("%20", " ");

            let response = ResponseBuilder::new()
                .status(200, "OK")
                .header("Content-Type", "text/plain")
                .body(echo_string.as_bytes());

            // Apply gzip middleware
            let response = middleware::gzip_response_middleware(request, response)?;

            Ok(response)
        }),
        ("GET /files/:filename", handle_read_file),
        ("POST /files/:filename", handle_post_file),
    ])?;

    server.run().await
}

fn handle_read_file(req_info: RequestInfo) -> Result<ResponseBuilder> {
    let request = req_info.request();
    let filename = request.params().get("filename").unwrap();

    let path = format!("{}/{}", req_info.pub_dir(), filename);

    let file = fs::read(path);

    let response = match file {
        Ok(file) => ResponseBuilder::new()
            .status(200, "OK")
            .header("Content-Type", "application/octet-stream")
            .body(&file),
        Err(_) => ResponseBuilder::new()
            .status(404, "Not Found")
            .header("Content-Type", "text/plain"),
    };

    Ok(response)
}

fn handle_post_file(req_info: RequestInfo) -> Result<ResponseBuilder> {
    let request = req_info.request();
    let filename = request.params().get("filename").unwrap();

    let path = format!("{}/{}", req_info.pub_dir(), filename);

    let file = fs::write(path, request.body().unwrap());

    let response = match file {
        Ok(_) => ResponseBuilder::new()
            .status(201, "Created")
            .header("Content-Type", "text/plain"),
        Err(_) => ResponseBuilder::new()
            .status(500, "Internal Server Error")
            .header("Content-Type", "text/plain")
            .body("Error writing file".as_bytes()),
    };

    Ok(response)
}
