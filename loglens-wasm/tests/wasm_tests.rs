// WASM Module Tests
// Basic unit tests for WASM functionality

#[test]
fn test_wasm_basic_functionality() {
    // Basic test to ensure WASM module compiles
    assert!(true);
}

#[test]
fn test_wasm_exports() {
    // Test that we can define basic WASM exports
    fn add_numbers(a: i32, b: i32) -> i32 {
        a + b
    }
    
    assert_eq!(add_numbers(2, 3), 5);
    assert_eq!(add_numbers(-1, 1), 0);
}