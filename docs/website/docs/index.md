# TopoHedral Tracing

`topohedral-tracing` provides lightweight, feature-gated diagnostics for the
TopoHedral Rust crates. It includes familiar logging macros, runtime filtering,
and indented function and scope traces.

Tracing is controlled in two stages:

1. the Cargo feature `enable_trace` includes tracing code in the build; and
2. the `TOPO_LOG` environment variable chooses which messages are printed at
   runtime.

With the feature disabled, calls to the logging macros are compiled out. This
makes it practical for libraries to keep detailed diagnostics in their source
without enabling them in ordinary builds.

## What the crate provides

- `trace!`, `debug!`, `info!`, `warn!`, and `error!` logging macros;
- exact-target and global runtime filters through `TOPO_LOG`;
- `#[trace_fn]` for automatic function entry, exit, and indentation;
- `trace_scope!` for tracing an arbitrary lexical scope; and
- per-thread indentation that makes nested calls easier to follow.

```rust
use topohedral_tracing::{info, trace, trace_fn};

#[trace_fn]
fn solve(iterations: usize) {
    info!("starting solver");
    trace!("iteration budget = {iterations}");
}

fn main() {
    topohedral_tracing::init().expect("tracing should initialize");
    solve(20);
}
```

Run a build that includes tracing and enable all levels:

```console
TOPO_LOG=all=trace cargo run --features enable_trace
```

Start with [Getting started](getting-started.md), then use the
[user guide](user-guide/index.md) for filtering, logging targets, and scope
tracing.
