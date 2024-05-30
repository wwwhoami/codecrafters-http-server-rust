use std::collections::HashMap;

use itertools::Itertools;
use regex::Regex;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};

use anyhow::Result;

use crate::{
    request::{HTTPError, Request},
    response::{Response, ResponseBuilder},
};

pub struct Route {
    path: String,
    params: HashMap<String, String>,
}

impl Route {
    pub fn new(path: &str) -> Self {
        let mut params = HashMap::new();

        path.split('/').for_each(|part| {
            if part.starts_with(':') {
                if let Some(param_key) = part.strip_prefix(':') {
                    params.insert(param_key.to_string(), String::new());
                }
            }
        });

        Route {
            path: path.to_string(),
            params,
        }
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn params(&self) -> &HashMap<String, String> {
        &self.params
    }

    pub fn add_param(&mut self, key: &str, value: &str) {
        self.params.insert(key.to_string(), value.to_string());
    }
}

type RouteHandlerFn = fn(Request) -> Result<Response>;
#[derive(Debug, Clone)]
pub struct RouteHandler {
    handler_fn: RouteHandlerFn,
    pattern: Regex,
    params: Vec<String>,
}

impl RouteHandler {
    pub fn new(handler: RouteHandlerFn, pattern: Regex, params: &[String]) -> Self {
        RouteHandler {
            handler_fn: handler,
            pattern,
            params: params.to_vec(),
        }
    }

    pub fn handler_fn(&self) -> RouteHandlerFn {
        self.handler_fn
    }

    pub fn pattern(&self) -> &Regex {
        &self.pattern
    }
}

type RouteHandlers = Vec<RouteHandler>;

#[derive(Debug)]
pub struct Server {
    listener: TcpListener,
    route_handlers: RouteHandlers,
}

impl Server {
    pub async fn new(host: &str, port: i32) -> Result<Server> {
        let addr = format!("{}:{}", host, port);
        let listener = TcpListener::bind(addr).await?;

        Ok(Server {
            listener,
            route_handlers: Vec::new(),
        })
    }

    pub fn add_route_handler(
        &mut self,
        path: &str,
        handler: fn(Request) -> Result<Response>,
    ) -> Result<()> {
        // Extract the params from the path
        let params = path
            .split('/')
            .filter_map(|part| {
                if part.starts_with(':') {
                    part.strip_prefix(':').map(|p| p.to_string())
                } else {
                    None
                }
            })
            .collect::<Vec<String>>();

        // Replace the params with regex patterns
        let pattern = path
            .split('/')
            .collect::<Vec<&str>>()
            .iter_mut()
            .map(|part| {
                if part.starts_with(':') {
                    *part = "([^/]+)"
                }
                *part
            })
            .join("/");
        // Add start and end anchors to the pattern to ensure it matches the entire path
        let pattern = format!("^{}$", pattern.replace('/', "\\/"));
        let pattern = Regex::new(&pattern)?;

        println!(
            "Path: {:?}, Pattern: {:?}, Params: {:?}",
            path, pattern, params
        );

        let route_handler = RouteHandler::new(handler, pattern, &params);

        self.route_handlers.push(route_handler);

        Ok(())
    }

    pub fn route_handlers(&mut self, handlers: &[(&str, RouteHandlerFn)]) -> Result<()> {
        for (path, handler) in handlers {
            self.add_route_handler(path, *handler)?;
        }

        Ok(())
    }

    pub async fn run(self) -> Result<()> {
        println!(
            "server listening on port {}",
            self.listener.local_addr()?.port()
        );

        loop {
            let (stream, _) = self.listener.accept().await?;

            let route_handler = self.route_handlers.clone();

            tokio::spawn(async move {
                let mut handler = Handler::new(stream, route_handler);
                let _ = handler.handle().await;
            });
        }
    }
}

pub struct Handler {
    tcp_stream: TcpStream,
    route_handlers: RouteHandlers,
}

impl Handler {
    pub fn new(stream: TcpStream, route_handlers: RouteHandlers) -> Handler {
        Handler {
            tcp_stream: stream,
            route_handlers,
        }
    }

    pub async fn handle(&mut self) -> Result<()> {
        let request = self.read_request().await;

        let response = match request {
            Ok(mut request) => {
                // Find the handler that matches the request's path
                let handler = self.route_handlers.iter().find(|handler| {
                    let path = request.request_line().path();
                    let mut params = HashMap::new();

                    // If the path matches the handler's pattern, extract the params
                    if handler.pattern().is_match(path) {
                        let captures = handler.pattern().captures(path).unwrap();

                        // For each param in the handler's pattern, extract the value from the path
                        for (i, name) in handler.params.iter().enumerate() {
                            // The first capture group is the entire match, so we start at 1
                            let value = captures.get(i + 1).unwrap().as_str();
                            // Replace %20 with a space
                            let value = value.replace("%20", " ");
                            params.insert(name.to_string(), value.to_string());
                        }

                        request.add_params(params);

                        println!("{}", request);

                        true
                    } else {
                        false
                    }
                });

                // Call the handler if it exists, otherwise return a 404
                match handler {
                    Some(handler) => (handler.handler_fn())(request),
                    None => ResponseBuilder::new().status(404, "Not Found").build(),
                }
            }
            Err(e) => {
                eprintln!("HTTP Error: {:?}", e);

                let (code, reason) = match e {
                    HTTPError::IoError(_) => (500, "Internal Server Error"),
                    HTTPError::IllegalMethod => (400, "Bad Request"),
                    HTTPError::Other(_) => (400, "Bad Request"),
                };

                ResponseBuilder::new().status(code, reason).build()
            }
        };

        let response = response.unwrap_or_else(|e| {
            eprintln!("Error: {}", e);
            ResponseBuilder::new()
                .status(500, "Internal Server Error")
                .build()
                .unwrap()
        });

        self.tcp_stream.write_all(&response.as_bytes()).await?;

        Ok(())
    }

    async fn read_request(&mut self) -> Result<Request, HTTPError> {
        let buffer = self.read_from_stream().await?;

        let str_buffer = String::from_utf8(buffer.to_vec())?;

        Request::parse_request(&str_buffer)
    }

    async fn read_from_stream(&mut self) -> Result<Vec<u8>, std::io::Error> {
        let mut buffer = Vec::new();

        loop {
            let mut buf = [0; 1024];
            let bytes_read = self.tcp_stream.read(&mut buf[..]).await?;

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
}
