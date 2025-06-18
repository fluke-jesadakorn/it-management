pub mod mocks;

#[allow(dead_code)]
pub fn setup() {
    // Common test setup code can go here
    std::env::set_var("RUST_LOG", "debug");
    let _ = env_logger::builder().is_test(true).try_init();
}

#[allow(dead_code)]
pub fn teardown() {
    // Common test cleanup code can go here
}
