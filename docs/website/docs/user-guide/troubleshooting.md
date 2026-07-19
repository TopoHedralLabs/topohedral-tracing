# Troubleshooting

## Nothing is printed

Check the three required gates in order:

1. **Compile-time feature:** run with `--features enable_trace` and confirm the
   calling crate declares and forwards that feature.
2. **Initialization:** call `topohedral_tracing::init()` before the first
   logging or scope-tracing call.
3. **Runtime filter:** set `TOPO_LOG` before initialization and choose a level
   that includes the message.

This command is a useful broad test:

```console
TOPO_LOG=all=trace cargo run --features enable_trace
```

## Some levels are missing

A filter level is a verbosity ceiling. For example, `all=debug` includes
`debug!`, `info!`, `warn!`, and `error!`, but not `trace!`. Use `all=trace` to
include every level.

## A module filter does not match

Targets are matched exactly; they are not prefixes. `my_app` does not match
`my_app::solver`.

Use the complete module path:

```console
TOPO_LOG=my_app::solver=debug cargo run --features enable_trace
```

Alternatively, give the message a custom target and filter that name:

```rust
topohedral_tracing::debug!(target: "solver", "trying candidate");
```

```console
TOPO_LOG=solver=debug cargo run --features enable_trace
```

Do not add spaces around commas or equals signs in `TOPO_LOG`.

## A target is noisier than requested

The `all` filter is a baseline. In this configuration:

```console
TOPO_LOG=all=debug,my_app::solver=error
```

`my_app::solver` still emits through `debug`, because an exact target cannot
narrow the global baseline. Remove or lower `all`, then list the targets that
should be enabled.

## Function entry and exit records are missing

`#[trace_fn]` and `trace_scope!` emit entry and exit at `info`. Ensure the
target's filter is `info`, `debug`, or `trace`.

Also confirm the attributed function is actually called after `init()`.

## Test output is hidden

The Rust test harness captures standard error. Run:

```console
TOPO_LOG=all=trace cargo test --features enable_trace -- --nocapture
```

When tests mutate `TOPO_LOG` or reinitialize tracing, avoid running those tests
concurrently because both the environment variable and logger configuration are
process-wide.

## Output contains ANSI colors

Set the standard `NO_COLOR` environment variable:

```console
NO_COLOR=1 TOPO_LOG=all=trace cargo run --features enable_trace
```

## Changing `TOPO_LOG` has no effect

The variable is parsed by `init()`, not on every message. Set it before process
startup. If application code changes the variable, it must reinitialize before
the new filters are observed; initializing once at startup is the recommended
pattern.
