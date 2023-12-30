/*
# https://docs.rs/axum/latest/axum/
cargo add tokio --features macros,rt-multi-thread
cargo add axum
cargo add --dev reqwest -F multipart
cargo add --dev http tower fastrand

curl -X POST http://localhost:3000/hello/name \
    -b tower.sid=abcd1234 -d "param1=value1&param2=value2"

curl -X POST http://localhost:3000/echo/post/json \
    -H "Content-Type: application/json" \
    -d '{"productId": 123456, "quantity": 100}'

curl -X POST http://localhost:3000/form-data/text \
    -F title='Cool story' -F year=2023 -F thumb=@demo.txt

curl -X POST http://localhost:3000/form-data/image \
    -F title='Cool story' -F year=2023 -F thumb=@demo.png

Demo png from https://pixabay.com/illustrations/hearts-love-art-cartoon-drawing-8342240/

*/

use tokio::net::TcpListener;

use axum::{
    body::Bytes,
    http::header::HeaderMap,
    http::{Method, Uri},
    Router,
};

// https://docs.rs/axum/latest/axum/extract/index.html#common-extractors
async fn echoreq(method: Method, uri: Uri, headers: HeaderMap, body: Bytes) -> String {
    // https://docs.rs/http/1.0.0/http/header/struct.HeaderMap.html
    let headers = headers
        .iter()
        .map(|(k, v)| {
            let v = v.to_str().unwrap_or_default();
            format!("{k}: {}", v)
        })
        .collect::<Vec<String>>()
        .join("\n");

    let body = match String::from_utf8(body.to_vec()) {
        Ok(body) => body,
        Err(_) => printable_bytes(body),
    };

    format!("{method} {uri}\n{headers}\n\n{body}")
}

fn router() -> Router {
    Router::new().fallback(echoreq)
}

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, router()).await.unwrap();
}

// TODO: eventually adapt some from std::core::str::validation::run_utf8_validation
fn printable_bytes(v: Bytes) -> String {
    let mut index = 0;
    let len = v.len();
    let mut out = vec!['.' as u8; len];

    while index < len {
        let oct = v[index];
        match oct {
            128.. => {
                // TODO: handle utf8 chars (oct >= 128), just validate them and copy bytes over -- must increment index extra
            }
            0..=8 | 11 | 12 | 14..=31 => {
                // just skip unprintable / invalid bytes (already '.')
            }
            _ => out[index] = oct,
        }
        index += 1;
    }

    unsafe { String::from_utf8_unchecked(out) }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::net::{SocketAddr, TcpListener as StdTcpListener};

    use reqwest::header::CONTENT_TYPE;
    use reqwest::multipart::{Form, Part};
    use reqwest::RequestBuilder;

    use tower::ServiceExt; // oneshot

    use axum::body::{to_bytes, Body};

    mod multipart;
    use multipart::{MultipartFieldValue, MultipartFields};

    const DEMO_IMG: &'static [u8] = include_bytes!("../demo.png");
    const DEMO_TXT: &'static [u8] = include_bytes!("../demo.txt");

    struct TestClient {
        addr: SocketAddr,
        client: reqwest::Client,
    }

    impl TestClient {
        fn new() -> Self {
            // https://github.com/tokio-rs/axum/blob/5d388c8d91440bc2865a894a3587971a4e7bc47c/examples/testing/src/main.rs#L129
            // https://github.com/tokio-rs/axum/blob/5d388c8d91440bc2865a894a3587971a4e7bc47c/axum/src/extract/multipart.rs#L334
            // https://github.com/tokio-rs/axum/blob/5d388c8d91440bc2865a894a3587971a4e7bc47c/axum/src/test_helpers/test_client.rs#L23

            // using std listener avoids async
            let std_listener = StdTcpListener::bind("0.0.0.0:0").unwrap();
            std_listener.set_nonblocking(true).unwrap();
            let listener = TcpListener::from_std(std_listener).unwrap();

            // let listener = TcpListener::bind("0.0.0.0:0").await.unwrap();
            let addr = listener.local_addr().unwrap();
            tokio::spawn(async move {
                axum::serve(listener, router()).await.unwrap();
            });

            let client = reqwest::Client::builder()
                .redirect(reqwest::redirect::Policy::none())
                .build()
                .unwrap();

            Self { addr, client }
        }

        fn addr(&self) -> &SocketAddr {
            &self.addr
        }

        async fn get(&self, url: &str) -> String {
            self.client
                .get(format!("http://{}{}", self.addr, url))
                .send()
                .await
                .unwrap()
                .text()
                .await
                .unwrap()
        }

        fn post(&self, url: &str) -> RequestBuilder {
            self.client.post(format!("http://{}{}", self.addr, url))
        }

        async fn send(req: RequestBuilder) -> String {
            req.send().await.unwrap().text().await.unwrap()
        }
    }

    #[tokio::test]
    async fn get() {
        let client = TestClient::new();
        let addr = client.addr();
        let path = "/home";
        let resp = client.get(path).await;
        assert_eq!(resp, format!("GET {path}\naccept: */*\nhost: {addr}\n\n"));
    }

    #[tokio::test]
    async fn post_simple() {
        let client = TestClient::new();
        let addr = client.addr();

        let path = "/hello/name";
        let req = client
            .post(path)
            .header("cookie", "tower.sid=abcd1234")
            .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
            .body("param1=value1&param2=value2");

        let resp = TestClient::send(req).await;
        assert_eq!(
            resp,
            format!(
                "POST {path}
cookie: tower.sid=abcd1234
content-type: application/x-www-form-urlencoded
accept: */*
host: {addr}
content-length: 27

param1=value1&param2=value2"
            )
        );
    }

    #[tokio::test]
    async fn post_json() {
        let client = TestClient::new();
        let addr = client.addr();

        let path = "/echo/post/json";
        let req = client
            .post(path)
            .header(CONTENT_TYPE, "application/json")
            .body("{\"productId\": 123456, \"quantity\": 100}");

        let resp = TestClient::send(req).await;
        assert_eq!(
            resp,
            format!(
                "POST {path}
content-type: application/json
accept: */*
host: {addr}
content-length: 38

{{\"productId\": 123456, \"quantity\": 100}}"
            )
        );
    }

    #[tokio::test]
    async fn post_formdata_text() {
        let client = TestClient::new();
        let addr = client.addr();

        let form = Form::new()
            .text("title", "Cool story")
            .text("year", "2023")
            .part(
                "thumb",
                Part::bytes(DEMO_TXT)
                    .file_name("demo.txt")
                    .mime_str("text/plain")
                    .unwrap(),
            );
        let boundary = form.boundary().to_owned();

        let path = "/form-data/text";
        let req = client.post(path).multipart(form);

        let resp = TestClient::send(req).await;
        assert_eq!(
            resp,
            format!(
                "POST {path}
content-type: multipart/form-data; boundary={boundary}
content-length: 513
accept: */*
host: {addr}

--{boundary}\r
Content-Disposition: form-data; name=\"title\"\r
\r
Cool story\r
--{boundary}\r
Content-Disposition: form-data; name=\"year\"\r
\r
2023\r
--{boundary}\r
Content-Disposition: form-data; name=\"thumb\"; filename=\"demo.txt\"\r
Content-Type: text/plain\r
\r
hi there

a file
\r
--{boundary}--\r
"
            )
        );
    }

    #[tokio::test]
    async fn post_formdata_file() {
        let client = TestClient::new();
        let addr = client.addr();

        let form = Form::new()
            .text("title", "Cool story")
            .text("year", "2023")
            .part(
                "thumb",
                Part::bytes(DEMO_IMG)
                    .file_name("demo.png")
                    .mime_str("image/png")
                    .unwrap(),
            );
        let boundary = form.boundary().to_owned();

        let path = "/form-data/image";
        let req = client.post(path).multipart(form);

        let resp = TestClient::send(req).await;
        assert_eq!(
            resp,
            format!(
                "POST {path}
content-type: multipart/form-data; boundary={boundary}
content-length: 791
accept: */*
host: {addr}

--{boundary}\r
Content-Disposition: form-data; name=\"title\"\r
\r
Cool story\r
--{boundary}\r
Content-Disposition: form-data; name=\"year\"\r
\r
2023\r
--{boundary}\r
Content-Disposition: form-data; name=\"thumb\"; filename=\"demo.png\"\r
Content-Type: image/png\r
\r
.PNG\r
.
...\rIHDR.....................sRGB.........gAMA......a....\tpHYs...t...t..f.x....IDAT(S..A..p.....3(.'.V.RZ....J.a8)....Vn.b..\\......P.?...O==..^...,3.....;........R..=.S....Mr...2.K...X(.l.D..a...v......q.Nk...xWf.^n
:.7..#J.0.....(.l.d5...1.........I`..t.X.g..k..-.......!..r.....IEND.B`.\r
--{boundary}--\r
"
            )
        );
    }

    #[tokio::test]
    async fn request_conversion() {
        // reqwest uses a custom random generator:
        // https://github.com/seanmonstar/reqwest/blob/4f54ba732f80ccb89e50954a369d6e8bb46375f2/src/async_impl/multipart.rs#L515
        // https://github.com/seanmonstar/reqwest/blob/4f54ba732f80ccb89e50954a369d6e8bb46375f2/src/util.rs#L26
        let mut rng = fastrand::Rng::new();
        let boundary = format!(
            "{:016x}-{:016x}-{:016x}-{:016x}",
            rng.u64(..),
            rng.u64(..),
            rng.u64(..),
            rng.u64(..)
        );

        let fields = MultipartFields::new(&[
            ("title", MultipartFieldValue::Text("Cool story")),
            ("year", MultipartFieldValue::Text("2023")),
            (
                "thumb",
                MultipartFieldValue::File {
                    filename: "demo.png",
                    data: DEMO_IMG,
                    content_type: "image/png",
                },
            ),
        ]);

        let path = "/form-data/image";
        let addr = "0.0.0.0:40123";
        let req_body = fields.to_http(&boundary);
        let request = http::Request::builder()
            .method("POST")
            .uri(path)
            .header(
                CONTENT_TYPE.to_string(),
                format!("multipart/form-data; boundary={boundary}"),
            )
            .header("content-length", req_body.len())
            .header("accept", "*/*")
            .header("host", addr)
            .body(Body::from(req_body))
            .unwrap();

        let response = router().oneshot(request).await.unwrap();
        let body_bytes = to_bytes(response.into_body(), 1234).await.unwrap().to_vec();
        let resp = unsafe { String::from_utf8_unchecked(body_bytes) };

        assert_eq!(
            resp,
            format!(
                "POST {path}
content-type: multipart/form-data; boundary={boundary}
content-length: 791
accept: */*
host: {addr}

--{boundary}\r
Content-Disposition: form-data; name=\"title\"\r
\r
Cool story\r
--{boundary}\r
Content-Disposition: form-data; name=\"year\"\r
\r
2023\r
--{boundary}\r
Content-Disposition: form-data; name=\"thumb\"; filename=\"demo.png\"\r
Content-Type: image/png\r
\r
.PNG\r
.
...\rIHDR.....................sRGB.........gAMA......a....\tpHYs...t...t..f.x....IDAT(S..A..p.....3(.'.V.RZ....J.a8)....Vn.b..\\......P.?...O==..^...,3.....;........R..=.S....Mr...2.K...X(.l.D..a...v......q.Nk...xWf.^n
:.7..#J.0.....(.l.d5...1.........I`..t.X.g..k..-.......!..r.....IEND.B`.\r
--{boundary}--\r
"
            )
        );
    }
}
