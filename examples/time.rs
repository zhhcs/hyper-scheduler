static NUM: std::sync::atomic::AtomicI32 = std::sync::atomic::AtomicI32::new(0);

fn main() {
    let start = std::time::Instant::now();
    do_some_add();
    do_some_sub();
    let end = std::time::Instant::now();
    println!("time cost: {:?}", end - start);
}

fn do_some_add() {
    for _ in 0..5_000_000 {
        NUM.fetch_add(2, std::sync::atomic::Ordering::Acquire);
    }
}

fn do_some_sub() {
    for _ in 0..5_000_000 {
        NUM.fetch_sub(2, std::sync::atomic::Ordering::Acquire);
    }
}
