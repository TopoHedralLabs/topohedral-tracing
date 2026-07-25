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

The name is anything convertible into `Cow<'static, str>`, so a string literal
costs no allocation and a computed name uses `format!`:

```rust
for block in 0..3 {
    topohedral_tracing::trace_scope!(format!("block {block}"));
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
TOPO_LOG=all=info cargo run --features trace
```

The scope target is the module containing the attributed function or macro
call, and prefix matching works in the same way as for ordinary log messages.

## Manual indentation

For unusual control flow, `increment_indent()` and `decrement_indent()` adjust
indentation without emitting entry or exit records:

```rust
use topohedral_tracing::{decrement_indent, increment_indent, info};

info!("outer");
increment_indent();
info!("inner");
decrement_indent();
info!("outer again");
```

`indent_level()` returns the current thread's depth.

Prefer `#[trace_fn]` or `trace_scope!` when possible. Manual calls must remain
balanced across every return and error path; `decrement_indent()` saturates at
zero rather than wrapping, so an unbalanced pair flattens output instead of
producing absurd indentation.

Guards can also be constructed directly, which requires naming the filter target
explicitly because a function cannot read its caller's `module_path!()`:

```rust
use topohedral_tracing::IndentGuard;

let _guard = IndentGuard::new(module_path!(), "a named scope");
```

## Threads and asynchronous code

Indentation is tracked independently for each operating-system thread, so
ordinary nested calls on different threads do not affect one another.

An asynchronous task can resume on a different worker thread after an `.await`.
A scope guard held across that migration would lower a different thread's
indentation than it raised, so `IndentGuard` is deliberately **not** `Send`: a
future holding one across an `.await` is itself `!Send` and cannot be spawned
onto a work-stealing executor. Keep traced scopes off the `.await` path when
reliable indentation is required.

Concurrent tasks that share a thread still interleave, because indentation is a
property of the thread rather than of the task.
