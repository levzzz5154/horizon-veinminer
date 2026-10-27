fn main() {
    println!(
        "{}",
        serde_json::to_string_pretty(&horizon_veinminer::Config::default()).unwrap()
    );
}
