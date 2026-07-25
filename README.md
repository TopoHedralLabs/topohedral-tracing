# topohedral-tracing

A [`log`](https://docs.rs/log) backend for the topohedral collection of crates, with two extras:

- a **compile-time switch** that erases trace points entirely, so instrumentation left in release
  builds costs nothing;
- **indentation-based call-stack tracing** via `trace_scope!` and `#[trace_fn]`.

Because it installs itself as the global `log` backend, records emitted by dependencies through
plain `log` macros are formatted and indented alongside your own. It provides five logging macros,
in decreasing order of verbosity: `trace!`, `debug!`, `info!`, `warn!`, `error!`.

```rust
use topohedral_tracing::{info, trace_fn};

#[trace_fn]
fn solve(input: u32) -> u32
{
    info!("solving for {input}");
    input * 2
}

fn main()
{
    topohedral_tracing::init().expect("tracing initialises once");
    solve(21);
}
```

```console
$ TOPO_LOG=all=trace cargo run --features trace
[INFO (1) main.rs:4]                    * Entering solve
[INFO (1) main.rs:6]                        solving for 21
[INFO (1) main.rs:4]                    * Leaving solve
```

## Usage

### Compile-time configuration

Trace points are only compiled in when the `trace` feature of **this** crate is enabled. Nothing is
required of your own crate:

```toml
[dependencies]
topohedral-tracing = { version = "0.2", registry = "cloudsmith", features = ["trace"] }
```

It is still common to forward it from a feature of your own so it can be toggled per build:

```toml
[features]
trace = ["topohedral-tracing/trace"]
```

Either form works. Enabling the feature anywhere in the dependency graph enables it for every
crate, and the constant `topohedral_tracing::ENABLED` reports which build you are in.

When the feature is off, the macro bodies are dead branches that the optimiser removes. Arguments
are still type-checked but never evaluated, so **do not put required work inside a log call**:

```rust
debug!("result = {}", update_state()); // update_state() does not run in a non-tracing build
```

The switch governs this crate's macros only. `init()` installs a working `log` backend in every
build, so a dependency's `log::warn!` still reaches the user of a non-tracing binary.

### Runtime configuration

Even with the feature on, the runtime filter prints nothing by default. Set `TOPO_LOG` before
calling `init()`:

```shell
export TOPO_LOG=<target>=<level>,<target>=<level>,...
```

Levels are `off`, `error`, `warn`, `info`, `debug`, `trace`, or the numbers `0`–`5`. A directive
without a level defaults to `info`. Each level includes the less verbose ones below it.

Targets default to the calling module's path and are matched by **module-path prefix**, longest
match winning. The special target `all` sets the level used when nothing else matches:

```shell
export TOPO_LOG=all=debug                        # everything at debug
export TOPO_LOG=all=warn,my_app::solver=trace    # quiet, except one module
export TOPO_LOG=all=debug,my_app::solver=warn    # verbose, except one noisy module
```

For filters that do not come from the environment, output somewhere other than stderr, or forced
colour, use `Builder` instead of `init()`.

### Macro name collisions

The exported macro names collide with `log`'s and `tracing`'s by design. Glob-importing both is
ambiguous — import the ones you want by name, or qualify at the call site as
`topohedral_tracing::debug!(...)`.

## Documentation

Full documentation is in `docs/website`. See `CHANGELOG.md` for the semver policy and the
0.1 → 0.2 migration notes, which include breaking changes to `TOPO_LOG` semantics.

## Development

```console
cargo test                          # tracing compiled out (the default consumers get)
cargo test --features trace         # tracing compiled in
cargo run --example basic --features trace
```

`tests/consumer-fixture` is a separate package that depends on this one the way a real downstream
crate does. It is the only configuration that can catch the compile-time switch leaking into the
consumer's feature set, so CI builds and runs it both ways.
