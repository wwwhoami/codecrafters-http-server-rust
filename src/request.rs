use std::collections::HashMap;

#[derive(Debug)]
pub enum HTTPError {
    IoError(std::io::Error),
    IllegalMethod,
    Other(String),
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

impl Request {
    pub fn parse_request(request: &str) -> Result<Request, HTTPError> {
        let mut parts = request.split("\r\n").collect::<Vec<&str>>();

        // Remove the last empty string
        parts.pop();

        let request_line = parts.remove(0);
        let body = parts
            .pop()
            .map(|b| match b {
                "" => None,
                _ => Some(b.as_bytes().to_vec()),
            })
            .unwrap();
        let headers = parts;

        let request_line = RequestLine::parse_request_line(request_line)?;
        let headers = Request::parse_headers(headers);

        Ok(Request {
            request_line,
            headers,
            body,
        })
    }

    fn parse_headers(headers: Vec<&str>) -> HashMap<String, String> {
        let mut headers_map = HashMap::new();
        println!("{:?}", headers);

        for line in headers {
            let parts = line.split(": ").collect::<Vec<&str>>();
            headers_map.insert(parts[0].to_string(), parts[1].to_string());
        }

        headers_map
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
