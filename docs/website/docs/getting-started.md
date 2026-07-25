# Getting Started

This guide configures the package, adds a first trace, and runs it with tracing
enabled.

## Configure the registry

`topohedral-tracing` is distributed through the TopoHedral Labs Cloudsmith
registry. Add the registry to `.cargo/config.toml` in your project or Cargo
configuration directory:

```toml
[registries.cloudsmith]
index = "sparse+https://cargo.cloudsmith.io/topohedrallabs/topohedral/"
token = "env:CARGO_REGISTRIES_CLOUDSMITH_TOKEN"
credential-provider = "cargo:token"
```

Set `CARGO_REGISTRIES_CLOUDSMITH_TOKEN` to a token with access to the registry.
Do not commit the token to source control.

## Add the dependency

Add the crate and enable its `trace` feature in `Cargo.toml`:

```toml
[dependencies]
topohedral-tracing = { version = "0.2", registry = "cloudsmith", features = ["trace"] }
```

Your crate does not need a feature of its own. The switch is a constant of
`topohedral-tracing`, so enabling the feature anywhere in the dependency graph
enables tracing in every crate that uses it.

To toggle tracing per build rather than always compiling it in, forward it from
a feature of your own:

```toml
[dependencies]
topohedral-tracing = { version = "0.2", registry = "cloudsmith" }

[features]
default = []
trace = ["topohedral-tracing/trace"]
```

Only the crate that decides the build — usually the top-level binary — needs to
do this. Intermediate libraries can log without forwarding anything.

!!! note "Changed in 0.2.0"

    Before 0.2.0 the feature was named `enable_trace`, and every crate
    containing tracing calls had to declare and forward a feature of that exact
    name, because the macros tested the *calling* crate's features. That is no
    longer necessary or meaningful. `enable_trace` still works as an alias for
    `trace` in 0.2 and will be removed in 0.3.

## Initialize tracing

Call `init()` before code that may emit messages. It reads `TOPO_LOG`, builds
the logger, and installs it as the global `log` backend. Installing a global
logger is a one-shot operation, so call it exactly once; a second call returns
`Err(TraceInitError::AlreadyInitialized)` rather than reconfiguring the first.

```rust
use topohedral_tracing::{debug, info, trace_fn};

#[trace_fn]
fn double(value: i32) -> i32 {
    debug!("doubling {value}");
    value * 2
}

fn main() {
    topohedral_tracing::init().expect("tracing should initialize");

    let answer = double(21);
    info!("answer = {answer}");
}
```

`TOPO_LOG` is read when `init()` runs. Set it before starting the program.

## Build and run with tracing

Compile the feature and set a runtime filter:

```console
TOPO_LOG=all=debug cargo run --features trace
```

The output is written to standard error and resembles:

```text
[INFO (1) src/main.rs:4]                * Entering double
[DEBUG(1) src/main.rs:6]                    doubling 21
[INFO (1) src/main.rs:4]                * Leaving double
[INFO (1) src/main.rs:14]               answer = 42
```

Source line numbers and spacing vary with the program. In a color-capable
terminal, the level name is colored.

## Verify the two switches

Try each mode to see how compile-time and runtime controls interact:

```console
# No tracing code is included.
cargo run

# Tracing is included, but the default runtime filter is off.
cargo run --features trace

# Tracing is included and messages through debug are enabled.
TOPO_LOG=all=debug cargo run --features trace
```

Next, see [Runtime configuration](user-guide/configuration.md) to select
individual module targets or [Scope tracing](user-guide/scope-tracing.md) to
trace nested calls.
