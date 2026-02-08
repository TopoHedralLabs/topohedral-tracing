//! Integration tests for the #[trace_fn] attribute macro.

use topohedral_tracing::{trace_fn, init};

#[trace_fn]
fn simple_function() -> i32 {
    42
}

#[trace_fn("custom_name")]
fn function_with_custom_name() -> &'static str {
    "hello"
}

#[trace_fn(name = "named_custom")]
fn function_with_name_eq() {
    // no-op
}

#[trace_fn]
fn function_with_args(x: i32, y: i32) -> i32 {
    x + y
}

#[trace_fn]
fn nested_outer() {
    nested_inner();
}

#[trace_fn]
fn nested_inner() {
    let _ = 1 + 1;
}

#[test]
fn test_trace_fn_basic() {
    std::env::set_var("TOPO_LOG", "all=5");
    init().unwrap();

    let result = simple_function();
    assert_eq!(result, 42);
}

#[test]
fn test_trace_fn_custom_name() {
    std::env::set_var("TOPO_LOG", "all=5");
    init().unwrap();

    let result = function_with_custom_name();
    assert_eq!(result, "hello");
}

#[test]
fn test_trace_fn_name_eq() {
    std::env::set_var("TOPO_LOG", "all=5");
    init().unwrap();

    function_with_name_eq();
}

#[test]
fn test_trace_fn_with_args() {
    std::env::set_var("TOPO_LOG", "all=5");
    init().unwrap();

    let result = function_with_args(3, 4);
    assert_eq!(result, 7);
}

#[test]
fn test_trace_fn_nested() {
    std::env::set_var("TOPO_LOG", "all=5");
    init().unwrap();

    nested_outer();
}
