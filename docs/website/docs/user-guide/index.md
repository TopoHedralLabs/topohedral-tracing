# User Guide

A message from this crate's macros is printed only when all three parts of
tracing are active:

1. `topohedral-tracing` is compiled with its `trace` feature;
2. `topohedral_tracing::init()` has run; and
3. `TOPO_LOG` enables the message's target and level.

Records emitted by a dependency through the `log` crate need only the last two:
`init()` installs a global `log` backend in every build.

This separation lets library authors add extensive diagnostics while leaving
the final decision to the application and its operator.

## Choose a topic

- [Runtime configuration](configuration.md) explains features, levels,
  targets, and `TOPO_LOG` filter syntax.
- [Logging messages](logging.md) covers the five logging macros, formatting,
  custom targets, and output structure.
- [Scope tracing](scope-tracing.md) covers `#[trace_fn]`, `trace_scope!`, and
  manual indentation.
- [Troubleshooting](troubleshooting.md) provides a checklist for missing or
  unexpected output.

## Typical application setup

Applications normally initialize once near the beginning of `main` and use
module-path targets supplied automatically by the macros:

```rust
use topohedral_tracing::{info, trace_fn};

#[trace_fn]
fn run_pipeline() {
    info!("pipeline ready");
}

fn main() {
    topohedral_tracing::init().expect("tracing should initialize");
    run_pipeline();
}
```

For broad diagnosis, run with `TOPO_LOG=all=trace`. Once the relevant area is
known, lower `all` and name the module or custom target of interest to reduce
noise, for example `TOPO_LOG=all=warn,my_app::solver=trace`.
