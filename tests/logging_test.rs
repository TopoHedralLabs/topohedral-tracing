//! Integration tests for logging macros, indentation, and trace_scope.

use topohedral_tracing::{
    debug, error, indent_dec, indent_inc, info, init, trace, trace_scope, warn,
};

#[test]
fn test_topo_log()
{
    println!("\n\n");
    std::env::set_var("TOPO_LOG", "all=5");
    init().unwrap();
    trace!("Hello, world! This is a test 1 {}", 5);
    trace!(target: "test",  "Hello, world! This is a test 2 {}", 5);
    debug!("Hello, world! This is a test 1 {}", 5);
    debug!(target: "test",  "Hello, world! This is a test 2 {}", 5);
    info!("Hello, world! This is a test 1 {}", 5);
    info!(target: "test",  "Hello, world! This is a test 2 {}", 5);
    warn!("Hello, world! This is a test 1 {}", 5);
    warn!(target: "test",  "Hello, world! This is a test 2 {}", 5);
    error!("Hello, world! This is a test 1 {}", 5);
    error!(target: "test",  "Hello, world! This is a test 2 {}", 5);

    println!("");
}

#[test]
fn test_indentation()
{
    println!("\n\n");
    std::env::set_var("TOPO_LOG", "all=5");
    init().unwrap();

    info!("Starting test");
    indent_inc();
    info!("Level 1");
    indent_inc();
    info!("Level 2");
    indent_inc();
    info!("Level 3");
    indent_dec();
    info!("Back to Level 2");
    indent_dec();
    info!("Back to Level 1");
    indent_dec();
    info!("Back to Level 0");

    println!("");
}

#[test]
fn test_trace_scope()
{
    println!("\n\n");
    std::env::set_var("TOPO_LOG", "all=5");
    init().unwrap();

    fn outer_function()
    {
        trace_scope!("outer_function");
        info!("Inside outer function");
        inner_function();
        info!("Back in outer function");
    }

    fn inner_function()
    {
        trace_scope!("inner_function");
        info!("Inside inner function");
        deepest_function();
    }

    fn deepest_function()
    {
        trace_scope!("deepest_function");
        info!("Inside deepest function");
    }

    info!("Test starting");
    outer_function();
    info!("Test complete");

    println!("");
}
