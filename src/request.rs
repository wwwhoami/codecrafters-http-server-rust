use std::{collections::HashMap, fmt::Display, string::FromUtf8Error};

#[derive(Debug)]
pub enum HTTPError {
    IoError(std::io::Error),
    IllegalMethod,
    Other(String),
}

impl From<std::io::Error> for HTTPError {
    fn from(value: std::io::Error) -> Self {
        HTTPError::IoError(value)
    }
}

impl From<FromUtf8Error> for HTTPError {
    fn from(value: FromUtf8Error) -> Self {
        HTTPError::Other(value.to_string())
    }
}

#[derive(Debug)]
pub enum HTTPMethod {
    GET,
    POST,
    PUT,
    PATCH,
    DELETE,
    HEAD,
    OPTIONS,
    CONNECT,
    TRACE,
}

impl HTTPMethod {
    fn parse_method(method: &str) -> Result<HTTPMethod, HTTPError> {
        match method {
            "GET" => Ok(HTTPMethod::GET),
            "POST" => Ok(HTTPMethod::POST),
            "PUT" => Ok(HTTPMethod::PUT),
            "PATCH" => Ok(HTTPMethod::PATCH),
            "DELETE" => Ok(HTTPMethod::DELETE),
            "HEAD" => Ok(HTTPMethod::HEAD),
            "OPTIONS" => Ok(HTTPMethod::OPTIONS),
            "CONNECT" => Ok(HTTPMethod::CONNECT),
            "TRACE" => Ok(HTTPMethod::TRACE),
            _ => Err(HTTPError::IllegalMethod),
        }
    }
}

#[derive(Debug)]
pub struct RequestLine {
    method: HTTPMethod,
    path: String,
    version: String,
}

impl RequestLine {
    fn parse_request_line(request_line: &str) -> Result<RequestLine, HTTPError> {
        let parts: Vec<&str> = request_line.split_whitespace().collect();

        if parts.len() != 3 {
            return Err(HTTPError::Other("Invalid request line".to_string()));
        }

        let method = HTTPMethod::parse_method(parts[0])?;
        let path = parts[1].to_string();
        let version = parts[2].to_string();

        Ok(RequestLine {
            method,
            path,
            version,
        })
    }

    pub fn method(&self) -> &HTTPMethod {
        &self.method
    }

    pub fn version(&self) -> &str {
        &self.version
    }

    pub fn path(&self) -> &str {
        &self.path
    }
}

#[derive(Debug)]
pub struct Request {
    request_line: RequestLine,
    headers: HashMap<String, String>,
    body: Option<Vec<u8>>,
}

impl Display for Request {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Request:\n\tMethod: {:?},\n\tPath: {},\n\tVersion: {},\n\tHeaders: {:?},\n\tBody: {:?}",
            self.request_line.method,
            self.request_line.path,
            self.request_line.version,
            self.headers,
            String::from_utf8(self.body.as_ref().unwrap_or(&Vec::new()).to_vec()).unwrap()
        )
    }
}

impl Request {
    pub fn parse_request(request: &str) -> Result<Request, HTTPError> {
        let parts = request.split("\r\n\r\n").collect::<Vec<&str>>();

        let headers = parts.first();
        let body = parts.get(1);

        // If we don't have both headers and body by splitting on \r\n\r\n
        // the request is malformed
        let (headers, body) = match (headers, body) {
            (Some(headers), Some(body)) => (*headers, *body),
            _ => {
                return Err(HTTPError::Other(
                    "Invalid request: malformed HTTP".to_string(),
                ))
            }
        };

        let mut headers = headers.split("\r\n").collect::<Vec<&str>>();
        // The first line is the request line
        let request_line = headers.remove(0);

        let request_line = RequestLine::parse_request_line(request_line)?;
        let headers = Request::parse_headers(headers);
        let body = Request::parse_body(body);

        Ok(Request {
            request_line,
            headers,
            body,
        })
    }

    fn parse_headers(headers: Vec<&str>) -> HashMap<String, String> {
        let mut headers_map = HashMap::new();

        for line in headers {
            let parts = line.split(": ").collect::<Vec<&str>>();
            headers_map.insert(parts[0].to_string(), parts[1].to_string());
        }

        headers_map
    }

    fn parse_body(body: &str) -> Option<Vec<u8>> {
        match body {
            "" => None,
            _ => Some(body.as_bytes().to_vec()),
        }
    }

    pub fn request_line(&self) -> &RequestLine {
        &self.request_line
    }

    pub fn headers(&self) -> &HashMap<String, String> {
        &self.headers
    }

    pub fn body(&self) -> Option<&Vec<u8>> {
        self.body.as_ref()
    }
}
