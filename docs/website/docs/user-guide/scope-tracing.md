# Scope Tracing

Scope tracing prints matching entry and exit records and indents messages
emitted between them. Entry and exit records use the `info` level.

## Trace a function with `#[trace_fn]`

Import and attach the attribute:

```rust
use topohedral_tracing::{debug, trace_fn};

#[trace_fn]
fn factorize(order: usize) {
    debug!("factorizing an order-{order} matrix");
}
```

By default, the Rust function name is used in the entry and exit records:

```text
* Entering factorize
    factorizing an order-4 matrix
* Leaving factorize
```

Two forms assign a custom display name:

```rust
use topohedral_tracing::trace_fn;

#[trace_fn("load configuration")]
fn load_config() {}

#[trace_fn(name = "assemble system")]
fn assemble() {}
```

The attribute preserves arguments and return values. It places a scope guard at
the start of the function body, so the exit record and indentation cleanup also
happen on early returns and during ordinary panic unwinding.

## Trace part of a function with `trace_scope!`

Use `trace_scope!` at the beginning of any lexical scope:

```rust
use topohedral_tracing::{debug, trace_scope};

fn process(values: &[f64]) {
    {
        trace_scope!("normalization");
        debug!("normalizing {} values", values.len());
    } // the leaving record is emitted here

    debug!("normalization complete");
}
```

The name is any value accepted by `{}` formatting:

```rust
for block in 0..3 {
    topohedral_tracing::trace_scope!(format_args!("block {block}"));
    // Work for this block is indented.
}
```

Place only one `trace_scope!` invocation in a lexical scope. The macro uses a
fixed internal guard name; use nested braces when several traced regions are
needed in one function.

## Filter scope records

Scope entry and exit records are `info` messages, not `trace` messages. A filter
must therefore include `info` or a more verbose level:

```console
TOPO_LOG=all=info cargo run --features enable_trace
```

The scope target is the module containing the attributed function or macro
call. Exact module filters work in the same way as ordinary log messages.

## Manual indentation

For unusual control flow, `indent_inc()` and `indent_dec()` adjust indentation
without emitting entry or exit records:

```rust
use topohedral_tracing::{indent_dec, indent_inc, info};

info!("outer");
indent_inc();
info!("inner");
indent_dec();
info!("outer again");
```

Prefer `#[trace_fn]` or `trace_scope!` when possible. Manual calls must remain
balanced across every return and error path.

## Threads and asynchronous code

Indentation is tracked independently for each operating-system thread, so
ordinary nested calls on different threads do not affect one another.

An asynchronous task can resume on a different worker thread after an `.await`.
A tracing scope held across that migration may therefore update indentation on
different threads. Keep traced scopes on one thread, or avoid holding them
across `.await`, when reliable indentation is required.
