# Developer Guide

The crate is a small workspace containing the runtime implementation and a
procedural-macro crate:

```text
src/lib.rs                         logging, filtering, and indentation
topohedral-tracing-macros/src/lib.rs  #[trace_fn] implementation
tests/                             integration and output-location tests
docs/website/                      this MkDocs site
```

## Local checks

Run the same broad checks used by continuous integration:

```console
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
```

Also test without `enable_trace` when changing an exported macro. Both sides of
each feature-gated expansion must remain valid:

```console
cargo test --all-targets
```

## Updating behavior

Keep the user documentation synchronized with these externally visible
contracts:

- the local and forwarded `enable_trace` feature;
- `TOPO_LOG` parsing and target matching;
- the level used for scope entry and exit records;
- source location, thread, color, and indentation in formatted output; and
- all accepted forms of `#[trace_fn]`.

Use integration tests for behavior that depends on expansion in a consuming
crate. In particular, source-location tests should assert that records point to
the caller rather than to `src/lib.rs`.

See [Building the documentation](building-docs.md) for site and Rust API
documentation commands.
