// https://doc.rust-lang.org/book/ch11-03-test-organization.html#submodules-in-integration-tests

pub enum MultipartFieldValue<'a> {
    Text(&'a str),
    File {
        filename: &'a str,
        data: &'a [u8],
        content_type: &'a str,
    },
}

pub struct MultipartFields<'a>(&'a [(&'a str, MultipartFieldValue<'a>)]);

impl<'a> MultipartFields<'a> {
    pub fn new(fields: &'a [(&'a str, MultipartFieldValue<'a>)]) -> Self {
        Self(fields)
    }

    pub fn to_http(&self, boundary: &str) -> Vec<u8> {
        let mut b = Vec::new();

        for (name, field) in self.0.iter() {
            b.extend_from_slice(
                format!("--{boundary}\r\nContent-Disposition: form-data; name=\"{name}\"")
                    .as_bytes(),
            );

            match field {
                MultipartFieldValue::Text(data) => {
                    b.extend_from_slice(format!("\r\n\r\n{data}\r\n").as_bytes())
                }
                MultipartFieldValue::File {
                    filename,
                    data,
                    content_type,
                } => b.extend_from_slice(
                    &[
                        format!(
                            "; filename=\"{filename}\"\r\nContent-Type: {content_type}\r\n\r\n"
                        )
                        .as_bytes(),
                        &data,
                        &[13u8, 10u8], // \r\n
                    ]
                    .concat(),
                ),
            }
        }

        b.extend_from_slice(format!("--{boundary}--\r\n").as_bytes());
        b
    }
}

pub fn multipart_request<'a>(
    path: &str,
    fields: MultipartFields<'a>,
    boundary: &str,
) -> (axum::http::request::Builder, Vec<u8>) {
    let body = fields.to_http(boundary);
    let request = axum::http::Request::builder()
        .method(axum::http::Method::POST)
        .uri(path)
        .header(
            "content-type",
            format!("multipart/form-data; boundary={boundary}"),
        )
        .header("content-length", body.len());

    (request, body)
}
