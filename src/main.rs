pub mod config;
pub mod request;
pub mod response;
pub mod server;

use std::{env, fs};

use anyhow::Result;
use config::Config;

use response::ResponseBuilder;
use server::Server;

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

            let response = ResponseBuilder::new()
                .status(200, "OK")
                .header("Content-Type", "text/plain")
                .body(echo_string.as_bytes())
                .build()?;

            Ok(response)
        }),
        ("/files/:filename", |req_info| {
            let request = req_info.request();

            let filename = request.params().get("filename").unwrap();

            let path = format!("{}/{}", req_info.pub_dir(), filename);
            println!("Path: {}", path);

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
        }),
    ])?;

    server.run().await
}
