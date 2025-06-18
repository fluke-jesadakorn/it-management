use it_management;

// Make common test utilities available
pub mod common;
pub mod command;
pub mod resolve_computer;

// Re-export the server module for tests
pub use it_management::server;

#[cfg(test)]
mod tests {
    #[test]
    fn test_lib_initialization() {
        // Test that our test lib setup works
        assert!(true);
    }
}
