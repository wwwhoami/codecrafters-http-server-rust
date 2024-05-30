pub mod request;
pub mod response;
pub mod server;

use anyhow::Result;
use response::ResponseBuilder;
use server::Server;

#[tokio::main]
async fn main() -> Result<()> {
    const HOST: &str = "127.0.0.1";
    const PORT: i32 = 4221;

    let mut server = Server::new(HOST, PORT).await?;

    server.route_handlers(&[
        ("/", |_| {
            let response = ResponseBuilder::new().status(200, "OK").build()?;
            Ok(response)
        }),
        ("/user-agent", |request| {
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
        ("/echo/:whatToEcho/:test", |request| {
            // let echo_string = request.request_line().path().replace("/echo/", "");
            let echo_string = request.params().get("whatToEcho").unwrap();
            let echo_string = echo_string.replace("%20", " ");

            let response = ResponseBuilder::new()
                .status(200, "OK")
                .header("Content-Type", "text/plain")
                .body(echo_string.as_bytes())
                .build()?;

            Ok(response)
        }),
    ])?;

    server.run().await
}
