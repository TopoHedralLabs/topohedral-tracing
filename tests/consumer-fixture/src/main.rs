//! Exercises `topohedral-tracing` the way a real downstream crate does.
//!
//! Run without features, tracing is compiled out and this prints only the summary line. Run with
//! `--features topohedral-tracing/trace` and `TOPO_LOG` set, it must also emit trace records —
//! *without* this crate declaring any feature of its own.

use topohedral_tracing::{debug, info, trace_fn, trace_scope, IndentGuard, ENABLED};

#[trace_fn]
fn traced_function(n: u32) -> u32
{
    info!("traced_function received {n}");
    inner_scope(n) * 2
}

fn inner_scope(n: u32) -> u32
{
    trace_scope!("inner_scope");
    debug!(target: "consumer_fixture::math", "doubling {n}");
    n + 1
}

fn manual_guard()
{
    let _guard = IndentGuard::new("consumer_fixture::manual", "manual_guard");
    info!(target: "consumer_fixture::manual", "inside the manual guard");
}

fn main()
{
    topohedral_tracing::init().expect("tracing initialises once");

    // A record emitted through the `log` facade, as any dependency would. It must be formatted by
    // the installed backend.
    log::info!("emitted via the log facade");

    let result = traced_function(20);
    manual_guard();

    // Printed on stdout so the harness can separate it from trace output on stderr.
    println!("enabled={ENABLED} result={result}");
    assert_eq!(result, 42);
}
