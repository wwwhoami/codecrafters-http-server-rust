use std::{collections::HashMap, fmt::Display};

#[derive(Debug)]
pub struct Status {
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

#[derive(Debug)]
pub struct Response {
    status: Status,
    headers: HashMap<String, String>,
    body: Vec<u8>,
}

impl Response {
    pub fn new(status: Status) -> Response {
        Response {
            status,
            headers: HashMap::new(),
            body: Vec::new(),
        }
    }

    pub fn status(&self) -> &Status {
        &self.status
    }

    pub fn headers(&self) -> &HashMap<String, String> {
        &self.headers
    }

    pub fn body(&self) -> &Vec<u8> {
        &self.body
    }

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

    pub fn set_status(&mut self, status: Status) {
        self.status = status;
    }

    pub fn set_headers(&mut self, headers: HashMap<String, String>) {
        self.headers = headers;
    }

    pub fn set_header(&mut self, key: &str, value: &str) {
        self.headers.insert(key.to_string(), value.to_string());
    }

    pub fn set_body(&mut self, body: &[u8]) {
        self.body = body.to_vec();
    }
}
