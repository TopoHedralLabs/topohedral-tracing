# TopoHedral Tracing

`topohedral-tracing` provides lightweight, feature-gated diagnostics for the
TopoHedral Rust crates. It includes familiar logging macros, runtime filtering,
and indented function and scope traces.

Tracing is controlled in two stages:

1. the Cargo feature `trace`, on `topohedral-tracing` itself, includes tracing
   code in the build; and
2. the `TOPO_LOG` environment variable chooses which messages are printed at
   runtime.

With the feature disabled, calls to the logging macros become dead branches that
the optimizer removes. This makes it practical for libraries to keep detailed
diagnostics in their source without enabling them in ordinary builds.

## What the crate provides

- `trace!`, `debug!`, `info!`, `warn!`, and `error!` logging macros;
- prefix-matched and global runtime filters through `TOPO_LOG`;
- `#[trace_fn]` for automatic function entry, exit, and indentation;
- `trace_scope!` for tracing an arbitrary lexical scope;
- per-thread indentation that makes nested calls easier to follow; and
- a global `log` backend, so records emitted by dependencies through the `log`
  crate are formatted and indented alongside your own.

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
TOPO_LOG=all=trace cargo run --features trace
```

Start with [Getting started](getting-started.md), then use the
[user guide](user-guide/index.md) for filtering, logging targets, and scope
tracing.
