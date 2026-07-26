# Troubleshooting

## Nothing is printed

Check the three required gates in order:

1. **Compile-time feature:** run with `--features trace`, or confirm that
   something in the dependency graph enables `topohedral-tracing/trace`. Print
   `topohedral_tracing::ENABLED` if in doubt.
2. **Initialization:** call `topohedral_tracing::init()` before the first
   logging or scope-tracing call, and check its result — if another `log`
   backend was installed first, it returns
   `Err(TraceInitError::AlreadyInitialized)` and nothing this crate emits will
   be formatted.
3. **Runtime filter:** set `TOPO_LOG` before initialization and choose a level
   that includes the message.

This command is a useful broad test:

```console
TOPO_LOG=all=trace cargo run --features trace
```

## Some levels are missing

A filter level is a verbosity ceiling. For example, `all=debug` includes
`debug!`, `info!`, `warn!`, and `error!`, but not `trace!`. Use `all=trace` to
include every level.

## A module filter does not match

Targets are matched by module-path prefix, so `my_app` selects `my_app::solver`.
Two things still prevent a match:

- **Boundaries are respected.** `my_app` does not select `my_apple`; only whole
  path segments count.
- **A more specific filter wins.** In `TOPO_LOG=my_app=trace,my_app::solver=off`
  the solver is silent, because the longest matching filter takes precedence.

Check the target a message actually carries: without an explicit `target:` it is
the full `module_path!()` of the call site, which for a message in `src/main.rs`
of a binary is the crate name.

## A target is noisier or quieter than expected

`all` is the level used when no other filter matches, and the longest matching
filter wins. So in:

```console
TOPO_LOG=all=debug,my_app::solver=error
```

`my_app::solver` emits only errors, while everything else emits `debug` and
below. If a target is unexpectedly verbose, look for a broader filter that also
matches it and add a more specific one.

## A `TOPO_LOG` entry is ignored

Unparseable directives are reported on standard error at startup, for example:

```text
topohedral-tracing: filter `solver=dbeug` has unknown level `dbeug`; expected one of off/error/warn/info/debug/trace or 0-5
```

The remaining directives still apply, so a typo silences one target rather than
all of them. Check for that line before assuming the filter is correct.

## Function entry and exit records are missing

`#[trace_fn]` and `trace_scope!` emit entry and exit at `info`. Ensure the
target's filter is `info`, `debug`, or `trace`.

Also confirm the attributed function is actually called after `init()`.

## Test output is hidden

The Rust test harness captures standard error. Run:

```console
TOPO_LOG=all=trace cargo test --features trace -- --nocapture
```

Note that filters are frozen when `init()` runs and a process has only one
global logger, so tests that each need a different `TOPO_LOG` cannot share a
process. Run each such case in a child process; `tests/common/mod.rs` in this
repository is a working harness for that pattern.

## Output contains ANSI colors

Color is enabled only when the output stream is a terminal, so redirected or
piped output is already plain. To force it off explicitly, set the standard
`NO_COLOR` environment variable:

```console
NO_COLOR=1 TOPO_LOG=all=trace cargo run --features trace
```

`Builder::color(false)` does the same programmatically, and
`Builder::color(true)` forces color on for a non-terminal sink.

## Changing `TOPO_LOG` has no effect

The variable is parsed by `init()`, not on every message. Set it before process
startup. Since `init()` is one-shot, a program cannot reinitialize to pick up a
new value; use `Builder::filters` if filters must be chosen programmatically.
