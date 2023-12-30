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

use echoreq::router;

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, router()).await.unwrap();
}
