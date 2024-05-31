use anyhow::{anyhow, Result};
use std::{collections::HashMap, fmt::Display};

#[derive(Debug)]
struct Status {
    code: u16,
    reason: String,
}

impl Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.code, self.reason)
    }
}

impl Status {
    pub fn new(code: u16, reason: &str) -> Status {
        Status {
            code,
            reason: reason.to_string(),
        }
    }
}
pub struct ResponseBuilder {
    status: Option<Status>,
    headers: HashMap<String, String>,
    body: Vec<u8>,
}

impl Default for ResponseBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl ResponseBuilder {
    pub fn new() -> ResponseBuilder {
        ResponseBuilder {
            status: None,
            headers: HashMap::new(),
            body: Vec::new(),
        }
    }

    pub fn status(mut self, code: u16, reason: &str) -> Self {
        let status = Status::new(code, reason);
        self.status = Some(status);
        self
    }

    pub fn header(mut self, key: &str, value: &str) -> Self {
        self.headers.insert(key.to_string(), value.to_string());
        self
    }

    pub fn headers(mut self, headers: &[(&str, &str)]) -> Self {
        let mut headers_map = HashMap::new();

        for (key, value) in headers {
            headers_map.insert(key.to_string(), value.to_string());
        }

        self.headers = headers_map;
        self
    }

    pub fn body(mut self, body: &[u8]) -> Self {
        self.body = body.to_vec();
        self
    }

    pub fn get_body(&self) -> &Vec<u8> {
        &self.body
    }

    pub fn build(self) -> Result<Response> {
        if let Some(status) = self.status {
            Ok(Response {
                status,
                headers: self.headers,
                body: self.body,
            })
        } else {
            Err(anyhow!("Cannot build Response without a status"))
        }
    }
}

#[derive(Debug)]
pub struct Response {
    status: Status,
    headers: HashMap<String, String>,
    body: Vec<u8>,
}

impl Response {
    pub fn as_bytes(&self) -> Vec<u8> {
        let mut response = format!("HTTP/1.1 {}\r\n", self.status);

        for (key, value) in &self.headers {
            response.push_str(&format!("{}: {}\r\n", key, value));
        }

        response.push_str(&format!("Content-Length: {}\r\n", self.body.len()));

        response.push_str("\r\n");

        let mut response_bytes = response.as_bytes().to_vec();
        response_bytes.append(&mut self.body.clone());

        response_bytes
    }
}
