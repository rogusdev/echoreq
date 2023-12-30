use std::net::{SocketAddr, TcpListener as StdTcpListener};

use reqwest::header::CONTENT_TYPE;
use reqwest::multipart::{Form, Part};
use reqwest::RequestBuilder;

use tokio::net::TcpListener;

use tower::ServiceExt; // oneshot

use axum::body::{to_bytes, Body};

mod multipart;
use multipart::{multipart_request, MultipartFieldValue, MultipartFields};

use echoreq::router;

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
    let (builder, req_body) = multipart_request(path, fields, &boundary);
    let request = builder
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
