pub mod config;
pub mod request;
pub mod response;
pub mod server;
pub mod utils;

use std::{env, fs};

use anyhow::Result;
use config::Config;

use request::HTTPMethod;
use response::{Response, ResponseBuilder};
use server::{RequestInfo, Server};
use utils::gzip_str;

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
        ("/", |_| {
            let response = ResponseBuilder::new().status(200, "OK").build()?;
            Ok(response)
        }),
        ("/user-agent", |req_info| {
            let request = req_info.request();

            let default_agent = "Unknown".to_string();

            let user_agent = request
                .headers()
                .get("User-Agent")
                .unwrap_or(&default_agent);

            let response = ResponseBuilder::new()
                .status(200, "OK")
                .header("Content-Type", "text/plain")
                .body(user_agent.as_bytes())
                .build()?;

            Ok(response)
        }),
        ("/echo/:whatToEcho", |req_info| {
            let request = req_info.request();

            let echo_string = request.params().get("whatToEcho").unwrap();
            let echo_string = echo_string.replace("%20", " ");

            let accept_encoding = request.headers().get("Accept-Encoding");

            if accept_encoding.is_some()
                && accept_encoding
                    .unwrap()
                    .split(',')
                    .map(|x| x.trim())
                    .any(|x| x == "gzip")
            {
                // Gzip the echo string
                let gzipped_echo = gzip_str(&echo_string)?;

                let response = ResponseBuilder::new()
                    .status(200, "OK")
                    .headers(&[("Content-Encoding", "gzip"), ("Content-Type", "text/plain")])
                    .body(&gzipped_echo)
                    .build()?;

                return Ok(response);
            }

            let response = ResponseBuilder::new()
                .status(200, "OK")
                .header("Content-Type", "text/plain")
                .body(echo_string.as_bytes())
                .build()?;

            Ok(response)
        }),
        ("/files/:filename", |req_info| {
            let request = req_info.request();

            let response = match request.method() {
                HTTPMethod::GET => handle_read_file(&req_info)?,
                HTTPMethod::POST => handle_post_file(&req_info)?,
                _ => ResponseBuilder::new()
                    .status(405, "Method Not Allowed")
                    .build()?,
            };

            Ok(response)
        }),
    ])?;

    server.run().await
}

fn handle_read_file(req_info: &RequestInfo) -> Result<Response> {
    let request = req_info.request();
    let filename = request.params().get("filename").unwrap();

    let path = format!("{}/{}", req_info.pub_dir(), filename);

    let file = fs::read(path);

    let response = match file {
        Ok(file) => ResponseBuilder::new()
            .status(200, "OK")
            .header("Content-Type", "application/octet-stream")
            .body(&file)
            .build()?,
        Err(_) => ResponseBuilder::new()
            .status(404, "Not Found")
            .header("Content-Type", "text/plain")
            .build()?,
    };

    Ok(response)
}

fn handle_post_file(req_info: &RequestInfo) -> Result<Response> {
    let request = req_info.request();
    let filename = request.params().get("filename").unwrap();

    let path = format!("{}/{}", req_info.pub_dir(), filename);

    let file = fs::write(path, request.body().unwrap());

    let response = match file {
        Ok(_) => ResponseBuilder::new()
            .status(201, "Created")
            .header("Content-Type", "text/plain")
            .build()?,
        Err(_) => ResponseBuilder::new()
            .status(500, "Internal Server Error")
            .header("Content-Type", "text/plain")
            .body("Error writing file".as_bytes())
            .build()?,
    };

    Ok(response)
}
