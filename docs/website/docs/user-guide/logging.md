# Logging Messages

The crate exports five macros with `format_args!`-style arguments:

```rust
use topohedral_tracing::{debug, error, info, trace, warn};

fn report(step: usize, residual: f64) {
    trace!("entered report");
    debug!("step {step}, residual {residual:.3e}");
    info!("iteration {step} complete");
    warn!("example warning at step {step}");
    error!("example error at step {step}");
}
```

The levels, from most to least verbose, are `trace`, `debug`, `info`, `warn`,
and `error`. A `debug` runtime filter includes debug, info, warn, and error
messages, but excludes trace messages.

## Default and custom targets

The usual form assigns the calling module path as the target:

```rust
topohedral_tracing::info!("mesh contains {} cells", cell_count);
```

Use the `target:` form when a stable, shorter, or cross-module category is more
useful:

```rust
topohedral_tracing::info!(target: "mesh", "loaded {} cells", cell_count);
topohedral_tracing::debug!(target: "solver", "residual = {residual:.3e}");
```

These messages can be selected independently:

```console
TOPO_LOG=mesh=info,solver=debug cargo run --features enable_trace
```

Targets affect filtering but are not printed in the formatted output.

## Output format

Messages are written to standard error. Each line contains:

```text
[LEVEL(thread) file:line]          indentation message
```

- `LEVEL` is colored by severity when terminal coloring is active;
- `thread` identifies the current operating-system thread;
- `file` is the source file name without its directory or extension;
- `line` is the macro call's source line; and
- four spaces are added for each active indentation level.

Set `NO_COLOR=1` when plain output is preferable, for example in a snapshot or
CI log:

```console
NO_COLOR=1 TOPO_LOG=all=debug cargo run --features enable_trace
```

## Formatting values

The macros accept the same interpolation and formatting syntax as standard Rust
formatting macros:

```rust
let iteration = 8;
let residual = 0.000_012_34;

topohedral_tracing::debug!(
    "iteration {iteration}: residual = {residual:.3e}"
);
```

With `enable_trace` disabled, logging arguments are compiled out and are not
evaluated. With the feature enabled, arguments may be evaluated even if the
runtime filter rejects the message, so avoid expensive diagnostic computation
unless it is acceptable in a tracing build.

## Using logs in tests

Rust's test harness captures output by default. Include the feature, set a
filter, and pass `--nocapture` when inspecting trace output:

```console
TOPO_LOG=all=trace cargo test --features enable_trace -- --nocapture
```
