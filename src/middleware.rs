use crate::{request::Request, response::ResponseBuilder, utils::gzip_str};

use anyhow::Result;

pub fn gzip_response_middleware(
    request: &Request,
    response: ResponseBuilder,
) -> Result<ResponseBuilder> {
    let accept_encoding = request.headers().get("Accept-Encoding");

    if accept_encoding.is_some()
        && accept_encoding
            .unwrap()
            .split(',')
            .map(|x| x.trim())
            .any(|x| x == "gzip")
    {
        let body = response.get_body();
        let compressed_body = gzip_str(String::from_utf8_lossy(body).as_ref())?;

        return Ok(response
            .header("Content-Encoding", "gzip")
            .body(&compressed_body));
    }

    Ok(response)
}
