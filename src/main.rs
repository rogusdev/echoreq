use tokio::net::TcpListener;

use echoreq::router;

const DEFAULT_PORT: usize = 3000;

#[tokio::main]
async fn main() {
    let port = match std::env::var("PORT") {
        Ok(s) => s.parse::<usize>().unwrap_or(DEFAULT_PORT),
        _ => DEFAULT_PORT,
    };

    println!("Listening on {port}");
    let listener = TcpListener::bind(&format!("0.0.0.0:{port}")).await.unwrap();
    axum::serve(listener, router()).await.unwrap();
}
