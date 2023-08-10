use std::thread;
use std::time::Duration;

use hyper_scheduler::axum::client::Client;
use hyper_scheduler::runwasm::Config;
// use rand::Rng;

// cargo build --release --package hyper-scheduler --example client
// sudo ./target/release/examples/client
#[tokio::main]
async fn main() {
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(tracing::Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
    tracing::info!("Starting");
    let client = Client::new();

    // let eet33 = 15;
    fib(&client, "task1", 400, 800, 40).await;
    client.call("http://127.0.0.1:3002/fib33").await.unwrap();
    // fib33(&client, "task2", eet33, 70).await;
    // thread::sleep(Duration::from_millis(5));
    fib(&client, "task3", 0, 0, 33).await;
    fib33(&client, "task4", 0, 0).await;
    fib(&client, "task5", 10, 25, 32).await;
    fib33(&client, "task6", 0, 0).await;
    fib(&client, "task7", 8, 20, 31).await;
    // thread::sleep(Duration::from_millis(5));
    // fib33(&client, "task8", eet33, 20).await;
    // fib(&client, "task9", 0, 0, 34).await;
    thread::sleep(Duration::from_millis(30));
    client.call("http://127.0.0.1:3002/fib33").await.unwrap();
    client.call("http://127.0.0.1:3002/fib33").await.unwrap();
    // for i in 1..10 {
    //     let _ = client
    //         .get_status_by_name(format!("task{}", i).as_str())
    //         .await;
    // }
    // let mut rng = rand::thread_rng();

    // for i in 200..300 {
    //     fib33(&client, format!("task{}0", i).as_str(), 0, 0).await;
    //     for j in 1..10 {
    //         fib(
    //             &client,
    //             format!("task{}{}", i, j).as_str(),
    //             0,
    //             0,
    //             rng.gen_range(30..35),
    //         )
    //         .await;
    //     }
    //     // thread::sleep(Duration::from_millis(rng.gen_range(120..500)));
    // }
}

async fn fib(client: &Client, name: &str, eet: u64, ddl: u64, param: i32) {
    let config = Config::new(
        &name,
        "/home/ubuntu/dev/hyper-scheduler/examples/fib().wasm",
        eet,
        ddl,
        "fib",
        Some("fib".to_owned()),
        Some(param),
    );
    client
        .call_with(&config, "http://127.0.0.1:3001/fib/config")
        .await
        .unwrap();
}

async fn fib33(client: &Client, name: &str, eet: u64, ddl: u64) {
    let config = Config::new(
        &name,
        "/home/ubuntu/dev/hyper-scheduler/examples/fib33.wasm",
        eet,
        ddl,
        "fib33",
        Some("fib33".to_owned()),
        None,
    );
    client
        .call_with(&config, "http://127.0.0.1:3002/fib33/config")
        .await
        .unwrap();
}
