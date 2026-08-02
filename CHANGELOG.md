# Changelog

All notable changes to this project are documented here. The format is based on
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Semver policy

The following are part of the public API and change only with a major (pre-1.0: minor) bump:

- the names and shapes of the exported macros, `IndentGuard`, `Builder`, `init`, `TraceInitError`,
  `ENABLED`, and the indentation functions;
- the **auto traits** of public types — in particular `IndentGuard` is deliberately `!Send`;
- the `trace` feature name;
- the major version of the `log` crate, whose `Level`/`Record` types this crate is built around.
  The `pub use log` re-export is `#[doc(hidden)]` and is macro plumbing, not API.

The `__private` module and every `#[doc(hidden)]` item may change in any release. `TOPO_LOG`
syntax is treated as API; changes to its *semantics* are called out below.

## [0.3.0]

### Changed — breaking

- ANSI colour support is now optional behind the `color` feature. The default
  feature graph is colour-free and uses a plain-text formatting path.
- The deprecated `enable_trace` feature alias has been removed. Use `trace`.
- `Builder::color(true)` has no effect unless the `color` feature is enabled.

This release lets downstream numerical crates enable instrumentation without
bringing the MPL-2.0-licensed `colored` dependency into their default or trace
feature graphs.

## [0.2.0]

A correctness release. Two defects made the crate unusable as published, and fixing them properly
required breaking changes, so the API-guideline cleanup is bundled into the same version.

### Fixed

- **The compile-time switch was evaluated in the consumer's crate, not this one.** The logging
  macros expanded to `#[cfg(feature = "enable_trace")]`, and Cargo resolves `cfg` against the crate
  being compiled. A consumer whose feature state did not exactly match this crate's got a compile
  error (the two `IndentGuard::new` arities disagreed); one that never declared a feature of its
  own got `unexpected_cfgs` warnings and silence. The gate is now the [`ENABLED`] constant of this
  crate, and consumers need no feature of their own. Regression test:
  `tests/consumer-fixture`, built by CI in both configurations.
- **The logger was never installed.** `init()` had its `log::set_boxed_logger` call commented out,
  so records emitted by dependencies through plain `log` macros were silently dropped, while
  `log::set_max_level` was still called and mutated global state that another backend might rely
  on. This crate is now a real `log` backend.
- **The proc-macro crate was not path-linked.** `Cargo.toml` depended on
  `topohedral-tracing-macros` by registry version only, so the workspace built the local copy but
  the library linked the *published* one; no change to the macro source was ever exercised by
  `cargo test`.
- **Unbounded memory growth.** Indentation was a `HashMap<ThreadId, usize>` that gained an entry
  per thread and never lost one. It is now thread-local, which also removes the global mutex from
  the logging path entirely — and with it the poisoning hazard whereby one panic while logging
  turned every later log call, including those in `Drop`, into a panic.
- **`IndentGuard` was `Send`** despite indentation being per-thread, so a guard moved across
  threads lowered a different thread's indentation than it raised. It is now `!Send`.
- **Scope records were attributed to this crate.** `IndentGuard::new` expanded `module_path!()`
  inside the library, so every scope's target was `topohedral_tracing` and no `TOPO_LOG` filter
  could select it.
- Malformed `TOPO_LOG` directives were silently treated as `info`. They are now reported on stderr
  and skipped, so a typo neither disables logging nor aborts startup.
- Colour was chosen from stdout's terminal status while output went to stderr, and the level column
  was padded *after* colouring, so ANSI escapes counted towards the field width and coloured output
  did not line up with uncoloured output.
- Source locations showed the file *stem*, so `parser/mod.rs` and `codegen/mod.rs` both rendered as
  `mod`. The file name is kept in full, with its parent directory for `mod.rs`/`lib.rs`/`main.rs`.
- `#[trace_fn]` hardcoded `::topohedral_tracing`, breaking for consumers that rename the
  dependency. The path is now resolved from the consumer's manifest.

### Changed — breaking

- **Feature renamed `enable_trace` → `trace`.** `enable_trace` remains as an alias for this release
  and will be removed in 0.3. Consumers that forward it (`enable_trace = ["topohedral-tracing/enable_trace"]`)
  keep working; new code should use `trace`.
- **`TOPO_LOG` filter semantics now match `env_logger`.** Two changes:
  - Targets are matched by **module-path prefix**, not exactly. `my_app=debug` now also selects
    `my_app::solver`. Boundaries are respected, so it does not select `my_apple`. When several
    directives match, the longest wins.
  - `all` is now the **default** level rather than a floor, so a per-target directive can make a
    target *quieter*. `TOPO_LOG=all=debug,my_app::solver=error` previously emitted `solver` at
    debug; it now emits it at error.

  Migration: filters that listed full module paths to work around the lack of prefix matching can
  usually be shortened, and any filter that relied on `all` raising a quieter target must be
  rewritten.
- **`init()` is one-shot.** It installs a global logger, so a second call returns
  `Err(TraceInitError::AlreadyInitialized)` instead of silently discarding the existing filters and
  wiping indentation state. Tests that reconfigured tracing per test case need one process each;
  see `tests/common/mod.rs` for the pattern this repository uses.
- **`init()` returns `Result<(), TraceInitError>`** instead of `Result<(), log::SetLoggerError>`.
  The old error type was foreign, unreachable without adding a `log` dependency, and could never
  actually be returned.
- **`indent_inc` / `indent_dec` renamed** to `increment_indent` / `decrement_indent`.
- **`IndentGuard::new` now takes `(target, name)`** and accepts anything `Into<Cow<'static, str>>`
  rather than requiring an owned `String`. It has one signature in both configurations; previously
  its arity depended on the feature. `new_with_caller` is replaced by the hidden `with_location`.
- **`topo_log` and `get_filename` are no longer public.** They were macro plumbing that leaked
  `log::Level` — a type from a `#[doc(hidden)]` re-export — into a documented signature. The
  replacement lives in `__private` and is not API.
- Macro arguments are now type-checked even when the `trace` feature is off. They are still not
  *evaluated*, so this costs nothing at runtime, but a trace point that does not compile now fails
  every build rather than only tracing builds.
- `init()` installs a global `log` backend **in every build**, including one without the `trace`
  feature. The compile-time switch erases this crate's own macros; it cannot and does not erase a
  dependency's `log::` calls, so a non-tracing binary now shows its dependencies' warnings and
  errors subject to `TOPO_LOG`. Previously nothing was installed and those records were dropped.

### Added

- [`Builder`] for programmatic configuration: explicit filters, output to stdout or an arbitrary
  `Write` sink, and forced colour. `init()` is the environment-driven shorthand.
- `indent_level()`, returning the current thread's indentation depth.
- `ENABLED`, the public form of the compile-time switch.
- `examples/basic.rs`, and real doctests — every documented example was previously ```ignore``` and
  so was never compiled.
- Crate-level `#![deny(missing_docs)]`, `#![warn(missing_debug_implementations)]` and
  `#![warn(rust_2018_idioms)]`.
- Cargo metadata: `keywords`, `categories`, `documentation`, `homepage`, `rust-version` (1.70).

## [0.1.1]

- Indentation support and the `trace_scope!` macro.
- Corrected source locations for indent guards.

## [0.1.0]

- Initial release.

[`ENABLED`]: https://docs.rs/topohedral-tracing/latest/topohedral_tracing/constant.ENABLED.html
[`Builder`]: https://docs.rs/topohedral-tracing/latest/topohedral_tracing/struct.Builder.html
[0.3.0]: https://github.com/TopoHedralLabs/topohedral-tracing/compare/v0.2.0...v0.3.0
