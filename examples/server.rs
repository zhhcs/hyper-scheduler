use hyper_scheduler::axum::server::Server;

// cargo build --package hyper-scheduler --example server
// RUST_LOG=info sudo ./target/debug/examples/server
fn main() {
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(tracing::Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
    tracing::info!("Starting");

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    rt.block_on(async {
        Server::start().await;
    });
}
