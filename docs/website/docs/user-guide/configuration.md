# Runtime Configuration

Tracing has independent compile-time and runtime controls. Both must be enabled
before a message can appear.

## Compile-time feature

Enable the `trace` feature on the dependency:

```toml
[dependencies]
topohedral-tracing = { version = "0.2", registry = "cloudsmith", features = ["trace"] }
```

or forward it from a feature of your own so it can be selected per build:

```toml
[dependencies]
topohedral-tracing = { version = "0.2", registry = "cloudsmith" }

[features]
default = []
trace = ["topohedral-tracing/trace"]
```

```console
cargo run --features trace
cargo test --features trace
```

The switch belongs to `topohedral-tracing`, not to the calling crate, so
enabling it anywhere in the dependency graph enables it everywhere. The constant
`topohedral_tracing::ENABLED` reports which build is running.

When the feature is not selected, the bodies of the logging macros are dead
branches that the optimizer removes. Their arguments are still type-checked, so
a trace point that no longer compiles is caught in every build, but they are
never evaluated.

The switch governs this crate's macros only. `init()` installs a working `log`
backend in every build, so records emitted by a dependency through the `log`
crate are still formatted and printed subject to `TOPO_LOG` — a non-tracing
binary keeps its dependencies' warnings and errors.

!!! warning "Do not put required work inside a log expression"

    Code such as `debug!("result = {}", update_state())` does not call
    `update_state()` in a build without `trace`. Logging arguments should only
    compute diagnostic values.

## Initialize before logging

Call `topohedral_tracing::init()` before any trace points execute:

```rust
fn main() {
    topohedral_tracing::init().expect("tracing should initialize");
    // Application startup continues here.
}
```

Initialization reads `TOPO_LOG` and installs the global `log` backend. Because a
process has exactly one global logger, `init()` is one-shot: filters are fixed
at that point, later changes to the environment have no effect, and a second
call returns `Err(TraceInitError::AlreadyInitialized)`.

For filters that do not come from the environment, output somewhere other than
standard error, or forced coloring, use `Builder`:

```rust
use topohedral_tracing::Builder;

Builder::new()
    .filters("all=warn,my_app::solver=trace")
    .color(false)
    .init()
    .expect("tracing should initialize");
```

## `TOPO_LOG` syntax

The variable is a comma-separated list of filters:

```text
<target>=<level>,<target>=<level>,...
```

A filter without an explicit level defaults to `info`:

```console
TOPO_LOG=my_app::solver cargo run --features trace
```

Whitespace around entries, targets and levels is ignored. A directive that
cannot be parsed — an unknown level, an empty target, more than one `=` — is
reported on standard error and skipped; the rest of the filter still applies.

### Levels

Each filter enables its level and every less-verbose level below it.

| Name | Number | Messages included |
| --- | ---: | --- |
| `trace` | `5` | trace, debug, info, warn, error |
| `debug` | `4` | debug, info, warn, error |
| `info` | `3` | info, warn, error |
| `warn` | `2` | warn, error |
| `error` | `1` | error only |
| `off` | `0` | none |

Names are recommended for readability, but the numeric forms are equivalent.
Without `TOPO_LOG`, all runtime logging is off.

### Targets and prefix matching

Without an explicit `target:`, each macro uses Rust's `module_path!()` at its
call site. For example, a message in module `solver` of crate `my_app` has the
target `my_app::solver`.

Targets are matched by **module-path prefix**. A filter for `my_app` selects
`my_app`, `my_app::solver` and `my_app::solver::inner`. Matching respects `::`
boundaries, so `my_app` does not select an unrelated `my_apple`.

When several filters match, the **longest** — that is, the most specific — wins:

```console
TOPO_LOG=my_app=info,my_app::solver=trace cargo run --features trace
```

Here `my_app::solver` emits every level and the rest of `my_app` emits `info`
and below.

### The `all` target

`all` sets the level used when no other filter matches:

```console
TOPO_LOG=all=info cargo run --features trace
```

Because it is a default rather than a floor, an explicit filter can make a
target either more or less verbose than `all`:

```console
# verbose overall, quiet in one noisy module
TOPO_LOG=all=debug,my_app::solver=warn cargo run --features trace

# quiet overall, verbose in one module of interest
TOPO_LOG=all=warn,my_app::solver=trace cargo run --features trace
```

### Custom targets

A `target:` argument replaces the module path, which is useful for a short name
or a category that spans modules:

```rust
topohedral_tracing::debug!(target: "solver", "candidate accepted");
```

```console
TOPO_LOG=solver=debug cargo run --features trace
```

Custom targets are matched by the same prefix rule, so naming them with `::`
gives them a hierarchy: `target: "solver::refine"` is selected by
`TOPO_LOG=solver=debug`.

!!! note "Changed in 0.2.0"

    Filter semantics changed to match `env_logger`, which the syntax has always
    resembled. Previously targets were matched **exactly**, so `my_app` did not
    select `my_app::solver`; and `all` was a **floor**, so an explicit filter
    could only make a target more verbose than `all`, never less.

    Filters that listed full module paths only to work around the lack of
    prefix matching can usually be shortened. Any filter that relied on `all`
    raising a quieter target must be rewritten.
