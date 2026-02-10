//! Integration tests for the #[trace_fn] attribute macro.

use topohedral_tracing::{init, trace, trace_fn};

#[trace_fn]
fn simple_function() -> i32
{
    trace!("print statement 1");
    42
}

#[trace_fn("custom_name")]
fn function_with_custom_name() -> &'static str
{
    trace!("print statement 2");
    "hello"
}

#[trace_fn(name = "named_custom")]
fn function_with_name_eq()
{
    trace!("print statement 3");
    // no-op
}

#[trace_fn]
fn function_with_args(
    x: i32,
    y: i32,
) -> i32
{
    trace!("print statement 4");
    x + y
}

#[trace_fn]
fn nested_outer()
{
    trace!("print statement 5");
    nested_inner();
}

#[trace_fn]
fn nested_inner()
{
    trace!("print statement 6");
    let _ = 1 + 1;
}

#[test]
fn test_trace_fn_basic()
{
    println!("\n\n");
    std::env::set_var("TOPO_LOG", "all=5");
    init().unwrap();

    let result = simple_function();
    assert_eq!(result, 42);
    println!("\n\n");
}

#[test]
fn test_trace_fn_custom_name()
{
    println!("\n\n");
    std::env::set_var("TOPO_LOG", "all=5");

    init().unwrap();

    let result = function_with_custom_name();
    assert_eq!(result, "hello");
    println!("\n\n");
}

#[test]
fn test_trace_fn_name_eq()
{
    println!("\n\n");
    std::env::set_var("TOPO_LOG", "all=5");
    init().unwrap();
    function_with_name_eq();
    println!("\n\n");
}

#[test]
fn test_trace_fn_with_args()
{
    println!("\n\n");
    std::env::set_var("TOPO_LOG", "all=5");
    init().unwrap();

    let result = function_with_args(3, 4);
    assert_eq!(result, 7);
    println!("\n\n");
}

#[test]
fn test_trace_fn_nested()
{
    println!("\n\n");
    std::env::set_var("TOPO_LOG", "all=5");
    init().unwrap();
    nested_outer();
    println!("\n\n");
}
