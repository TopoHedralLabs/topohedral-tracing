# Runtime Configuration

Tracing has independent compile-time and runtime controls. Both must be enabled
before a message can appear.

## Compile-time feature

Declare a local `enable_trace` feature and forward it to the dependency:

```toml
[dependencies]
topohedral-tracing = { version = "0.1.1", registry = "cloudsmith" }

[features]
default = []
enable_trace = ["topohedral-tracing/enable_trace"]
```

Then select it with Cargo:

```console
cargo run --features enable_trace
cargo test --features enable_trace
```

When the feature is not selected, the bodies of the logging macros are absent
from the build. Formatting expressions passed to those macros are not evaluated.

!!! warning "Do not put required work inside a log expression"

    Code such as `debug!("result = {}", update_state())` does not call
    `update_state()` in a build without `enable_trace`. Logging arguments should
    only compute diagnostic values.

## Initialize before logging

Call `topohedral_tracing::init()` before any trace points execute:

```rust
fn main() {
    topohedral_tracing::init().expect("tracing should initialize");
    // Application startup continues here.
}
```

Initialization reads `TOPO_LOG`. Changes to the environment variable after
that point do not alter the active filters unless the crate is initialized
again. A normal application should initialize once.

## `TOPO_LOG` syntax

The variable is a comma-separated list of filters:

```text
<target>=<level>,<target>=<level>,...
```

Do not put whitespace around entries, targets, or levels. A filter without an
explicit level defaults to `info`:

```console
TOPO_LOG=my_app::solver cargo run --features enable_trace
```

### Levels

Each filter enables its level and every less-verbose level below it.

| Name | Number | Messages included |
| --- | ---: | --- |
| `trace` | `5` | trace, debug, info, warn, error |
| `debug` | `4` | debug, info, warn, error |
| `info` | `3` | info, warn, error |
| `warn` | `2` | warn, error |
| `error` | `1` | error only |

Names are recommended for readability, but the numeric forms are equivalent.
Without `TOPO_LOG`, all runtime logging is off.

### The `all` target

`all` sets the baseline level for every target:

```console
TOPO_LOG=all=info cargo run --features enable_trace
```

An exact target can make that target more verbose:

```console
TOPO_LOG=all=warn,my_app::solver=trace cargo run --features enable_trace
```

In this example, `my_app::solver` emits every level and all other targets emit
warnings and errors. The `all` level is a baseline: an exact filter cannot make
a target less verbose than `all`.

### Exact targets

Without an explicit `target:`, each macro uses Rust's `module_path!()` at its
call site. For example, a message in module `solver` of crate `my_app` normally
has the target `my_app::solver`.

Target matching is exact. A filter for `my_app` does not automatically match
`my_app::solver`. Use `all` for a global filter, list full module targets, or
assign a short custom target at the call site:

```rust
topohedral_tracing::debug!(target: "solver", "candidate accepted");
```

```console
TOPO_LOG=solver=debug cargo run --features enable_trace
```

Multiple exact filters can be combined:

```console
TOPO_LOG=my_app::parser=trace,my_app::solver=debug cargo run --features enable_trace
```
