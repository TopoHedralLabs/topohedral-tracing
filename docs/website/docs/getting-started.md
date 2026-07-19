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

## Add the dependency and feature

Add the crate and forward a feature named `enable_trace` in `Cargo.toml`:

```toml
[dependencies]
topohedral-tracing = { version = "0.1.1", registry = "cloudsmith" }

[features]
default = []
enable_trace = ["topohedral-tracing/enable_trace"]
```

The forwarding feature is significant. The exported logging macros test the
calling crate's `enable_trace` feature as well as enabling the dependency's
implementation. Use the same forwarding pattern at each crate boundary that
contains tracing calls.

## Initialize tracing

Call `init()` before code that may emit messages. The function reads
`TOPO_LOG`, creates the logger, and returns an error-compatible result:

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
TOPO_LOG=all=debug cargo run --features enable_trace
```

The output is written to standard error and resembles:

```text
[INFO (1) main:4]                 * Entering double
[DEBUG(1) main:6]                     doubling 21
[INFO (1) main:4]                 * Leaving double
[INFO (1) main:14]                answer = 42
```

Source line numbers and spacing vary with the program. In a color-capable
terminal, the level name is colored.

## Verify the two switches

Try each mode to see how compile-time and runtime controls interact:

```console
# No tracing code is included.
cargo run

# Tracing is included, but the default runtime filter is off.
cargo run --features enable_trace

# Tracing is included and messages through debug are enabled.
TOPO_LOG=all=debug cargo run --features enable_trace
```

Next, see [Runtime configuration](user-guide/configuration.md) to select
individual module targets or [Scope tracing](user-guide/scope-tracing.md) to
trace nested calls.
