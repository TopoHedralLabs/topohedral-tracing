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
TOPO_LOG=mesh=info,solver=debug cargo run --features trace
```

Targets affect filtering but are not printed in the formatted output.

## Records from dependencies

The crate installs itself as the global `log` backend, so a dependency that
knows nothing about it still has its records formatted and indented the same
way:

```rust
log::info!("emitted by a dependency");
```

Such records carry that dependency's module path as their target and are
filtered by the same `TOPO_LOG` rules.

## Output format

Messages are written to standard error by default; `Builder` can direct them to
standard output or to any `Write` sink. Each line contains:

```text
[LEVEL(thread) file:line]          indentation message
```

- `LEVEL` is colored by severity when the optional `color` feature is active;
- `thread` identifies the current operating-system thread;
- `file` is the source file name, with its parent directory for `mod.rs`,
  `lib.rs` and `main.rs` where the name alone would be ambiguous;
- `line` is the macro call's source line; and
- four spaces are added for each active indentation level.

With the `color` feature enabled, color is used only when the output stream is a
terminal, so redirected output is already plain. Set `NO_COLOR=1` to force it
off, for example in a snapshot test:

```console
NO_COLOR=1 TOPO_LOG=all=debug cargo run --features trace
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

With `trace` disabled, logging arguments are type-checked but never evaluated —
the whole call is a dead branch. With the feature enabled, arguments may be
evaluated even if the runtime filter rejects the message, so avoid expensive
diagnostic computation unless it is acceptable in a tracing build.

## Using logs in tests

Rust's test harness captures output by default. Include the feature, set a
filter, and pass `--nocapture` when inspecting trace output:

```console
TOPO_LOG=all=trace cargo test --features trace -- --nocapture
```

Filters are fixed when `init()` runs, and a process has one global logger, so
tests that each need different filters must run in separate processes. See
[Troubleshooting](troubleshooting.md#test-output-is-hidden).
