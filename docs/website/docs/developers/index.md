# Developer Guide

The crate is a small workspace containing the runtime implementation and a
procedural-macro crate:

```text
src/lib.rs                            logging, filtering, and indentation
topohedral-tracing-macros/src/lib.rs  #[trace_fn] implementation
examples/basic.rs                     runnable tour of the output
tests/common/mod.rs                   subprocess fixture harness
tests/consumer-fixture/               a separate package that consumes the crate
docs/website/                         this MkDocs site
```

The proc-macro crate is a workspace member **and** a path dependency. Both are
required: without the `path` key the library links the published copy from the
registry instead of the local source, and no local macro change is ever tested.

## Local checks

The compile-time switch produces two distinct builds, and the one without the
feature is what consumers get by default. Check both:

```console
cargo fmt --all -- --check

cargo clippy --all-targets -- -D warnings
cargo clippy --all-targets --features trace -- -D warnings

cargo test
cargo test --features trace

cargo doc --no-deps --features trace
cargo run --example basic --features trace
```

Avoid `--all-features`: it enables the deprecated `enable_trace` alias alongside
`trace` and hides whether the default build works.

## Testing

Installing a global `log` backend is one-shot and freezes the filters, so a test
needing a particular `TOPO_LOG` cannot share a process with another. Such tests
are written as *fixtures*: a `#[test]` that returns immediately unless a marker
environment variable is set, re-executed by the parent through the test binary
so its stderr can be captured. `tests/common/mod.rs` provides `run_fixture` and
a parser that turns output lines back into levels, locations and indentation
depths, so assertions cover the layout as well as the content.

`tests/consumer-fixture` is a separate package, deliberately outside the
workspace, that depends on the crate the way a real downstream user does and
declares no tracing feature of its own. It is the only configuration that can
catch the compile-time switch leaking into the consumer's feature set — a
workspace member always sees the workspace's own features — so CI builds it
warning-free with tracing off and asserts it emits records with tracing on.

## Updating behavior

Keep the user documentation synchronized with these externally visible
contracts:

- the `trace` feature and the `ENABLED` constant;
- `TOPO_LOG` parsing, prefix matching, and the meaning of `all`;
- the level used for scope entry and exit records;
- source location, thread, color, and indentation in formatted output;
- the `Builder` options; and
- all accepted forms of `#[trace_fn]`.

Record breaking changes and the semver policy in `CHANGELOG.md`. Behavior that
depends on expansion in a consuming crate belongs in an integration test; in
particular, source-location tests should assert that records point to the caller
rather than to `src/lib.rs`.

See [Building the documentation](building-docs.md) for site and Rust API
documentation commands.
