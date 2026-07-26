//! Tracing for the topohedral collection of crates.
//!
//! # Introduction
//!
//! This crate is a [`log`] backend with two extras: a **compile-time** switch that erases trace
//! points entirely, and indentation-based call-stack tracing. It installs itself as the global
//! [`log`] logger, so messages emitted by dependencies through plain [`log`] macros are formatted
//! and indented alongside your own.
//!
//! It provides five logging macros, listed in decreasing order of verbosity:
//!
//! - [`trace!`]
//! - [`debug!`]
//! - [`info!`]
//! - [`warn!`]
//! - [`error!`]
//!
//! plus [`trace_scope!`] and [`trace_fn`] for call-stack tracing.
//!
//! # Usage
//!
//! ```no_run
//! // in `main`, before anything that logs
//! topohedral_tracing::init().expect("tracing initialises once");
//! topohedral_tracing::info!("starting up");
//! ```
//!
//! ## Compile-time configuration
//!
//! The bodies of the logging macros are only compiled when the `trace` feature of **this** crate
//! is enabled. Consumers do not need to declare a matching feature of their own:
//!
//! ```toml
//! [dependencies]
//! topohedral-tracing = { version = "0.2", features = ["trace"] }
//! ```
//!
//! It is still common to forward it from a feature of your own so that it can be toggled per
//! build:
//!
//! ```toml
//! [features]
//! trace = ["topohedral-tracing/trace"]
//! ```
//!
//! Either form works, and enabling the feature anywhere in the dependency graph enables it for
//! every crate — see [`ENABLED`].
//!
//! ## Runtime configuration
//!
//! Even with the `trace` feature on, the runtime filter defaults to printing nothing. Set the
//! `TOPO_LOG` environment variable before calling [`init`]:
//!
//! ```shell
//! export TOPO_LOG=<target>=<level>,<target>=<level>,...
//! ```
//!
//! Targets are matched by **module path prefix**: a filter for `my_app` also matches
//! `my_app::solver`, and the longest matching filter wins. The special target `all` sets the
//! level used when nothing else matches, so to log everything at `debug`:
//!
//! ```shell
//! export TOPO_LOG=all=debug
//! ```
//!
//! and to quieten one noisy module below that baseline:
//!
//! ```shell
//! export TOPO_LOG=all=debug,my_app::solver=warn
//! ```
//!
//! For programmatic configuration — a custom output sink, filters not read from the environment —
//! use [`Builder`].
//!
//! ## Macro name collisions
//!
//! This crate deliberately exports macros named `trace!`, `debug!`, `info!`, `warn!` and `error!`,
//! which collide with the identically named macros in [`log`] and in `tracing`. Glob-importing
//! both (`use log::*; use topohedral_tracing::*;`) is ambiguous. Import the ones you want by name,
//! or qualify at the call site as `topohedral_tracing::debug!(...)`.
//!
//--------------------------------------------------------------------------------------------------

#![deny(missing_docs)]
#![warn(missing_debug_implementations)]
#![warn(rust_2018_idioms)]

//{{{ crate imports
pub use topohedral_tracing_macros::trace_fn;
//}}}
//{{{ std imports
use std::borrow::Cow;
use std::cell::Cell;
use std::error::Error;
use std::fmt;
use std::io::{self, IsTerminal, Write};
use std::marker::PhantomData;
use std::path::Path;
use std::sync::Mutex;
use std::thread;
//}}}
//{{{ dep imports
use colored::Colorize;
#[doc(hidden)]
pub use log;
use log::{Level, LevelFilter, Log, Metadata, Record};
//}}}
//--------------------------------------------------------------------------------------------------
//{{{ collection: compile-time switch
/// Whether trace points are compiled in, i.e. whether the `trace` feature of this crate is
/// enabled.
///
/// The logging macros expand to `if ENABLED { .. }`. Because this constant is evaluated when
/// *this* crate is compiled rather than when the calling crate is, enabling the feature anywhere
/// in the dependency graph enables it everywhere, and consumers need no feature of their own.
///
/// When it is `false` the branch is dead and is removed by the optimiser, so trace points cost
/// nothing. The arguments are still type-checked, which means a trace point that does not compile
/// is caught in every build rather than only in tracing builds.
///
/// The switch governs *this crate's* macros only. Records emitted through [`log`] directly — by a
/// dependency, or by your own code calling `log::warn!` — cannot be erased from someone else's
/// crate and are not meant to be: [`init`] installs a working backend in every build, so a
/// dependency's warnings and errors still reach the user of a non-tracing binary.
pub const ENABLED: bool = cfg!(feature = "trace");
//}}}
//{{{ collection: indentation
thread_local! {
    /// Indentation depth, per thread.
    ///
    /// This is deliberately thread-local rather than a map keyed by [`thread::ThreadId`]: a map
    /// would grow without bound in a process that spawns many short-lived threads, and it would
    /// force the installed logger to be mutable.
    static INDENT: Cell<usize> = const { Cell::new(0) };
}

/// Number of spaces one indentation level occupies.
const INDENT_WIDTH: usize = 4;

/// Column at which the message body starts, measured from the end of the `[level(tid) file:line]`
/// prefix. Longer prefixes push the body right rather than losing the separating space.
const MESSAGE_COLUMN: usize = 40;

/// Increase the indentation level of the current thread by one.
///
/// Subsequent log messages on this thread are indented one level further. Pair every call with a
/// matching [`decrement_indent`]; prefer [`trace_scope!`] or [`trace_fn`], which pair them for you
/// via [`IndentGuard`] and cannot leak on an early return or a panic.
pub fn increment_indent() {
    INDENT.with(|i| i.set(i.get().saturating_add(1)));
}

/// Decrease the indentation level of the current thread by one, saturating at zero.
///
/// See [`increment_indent`].
pub fn decrement_indent() {
    INDENT.with(|i| i.set(i.get().saturating_sub(1)));
}

/// Returns the current thread's indentation level.
pub fn indent_level() -> usize {
    INDENT.with(|i| i.get())
}
//}}}
//{{{ collection: errors
/// The error returned by [`init`] and [`Builder::init`].
#[derive(Debug)]
#[non_exhaustive]
pub enum TraceInitError {
    /// A global logger has already been installed, either by this crate or by another [`log`]
    /// backend in the same process.
    ///
    /// Installing a global logger is a one-shot operation, so this is the expected outcome of a
    /// second call rather than a transient failure.
    AlreadyInitialized,
}

impl fmt::Display for TraceInitError {
    fn fmt(
        &self,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        match self {
            Self::AlreadyInitialized => f.write_str("a global logger has already been installed"),
        }
    }
}

impl Error for TraceInitError {}
//}}}
//{{{ collection: filters
//{{{ struct: Filters
/// A parsed set of target filters.
#[derive(Debug, Clone)]
struct Filters {
    /// Explicit `target=level` directives, matched by module-path prefix.
    directives: Vec<(String, LevelFilter)>,
    /// Level used when no directive matches. Set by the `all` target.
    default: LevelFilter,
}
//}}}
//{{{ impl Filters
impl Filters {
    /// Parses a `TOPO_LOG`-syntax filter string.
    ///
    /// Unparseable directives are reported through `on_error` and then skipped, matching
    /// `env_logger`'s behaviour: one typo should not silently disable logging, nor abort startup.
    fn parse(
        spec: &str,
        mut on_error: impl FnMut(String),
    ) -> Self {
        let mut filters = Self {
            directives: Vec::new(),
            default: LevelFilter::Off,
        };

        for directive in spec.split(',').map(str::trim).filter(|d| !d.is_empty()) {
            let (target, level) = match directive.split_once('=') {
                Some((target, level_str)) => {
                    if level_str.contains('=') {
                        on_error(format!(
                            "filter `{directive}` has more than one `=`; expected `target=level`"
                        ));
                        continue;
                    }
                    match parse_level(level_str.trim()) {
                        Some(level) => (target.trim(), level),
                        None => {
                            on_error(format!(
                                "filter `{directive}` has unknown level `{}`; expected one of \
                                 off/error/warn/info/debug/trace or 0-5",
                                level_str.trim()
                            ));
                            continue;
                        }
                    }
                }
                // A bare target defaults to `info`, as documented.
                None => (directive, LevelFilter::Info),
            };

            if target.is_empty() {
                on_error(format!("filter `{directive}` has an empty target"));
                continue;
            }

            if target == "all" {
                filters.default = level;
            } else {
                filters.directives.push((target.to_string(), level));
            }
        }

        filters
    }

    /// The level in force for `target`.
    ///
    /// The longest matching directive wins, so `my_app::solver=warn` narrows `my_app=debug` for
    /// that subtree. When nothing matches, the `all` level applies.
    fn level_for(
        &self,
        target: &str,
    ) -> LevelFilter {
        self.directives
            .iter()
            .filter(|(filter, _)| target_matches(target, filter))
            .max_by_key(|(filter, _)| filter.len())
            .map(|(_, level)| *level)
            .unwrap_or(self.default)
    }

    /// The most verbose level any target can produce, used to set [`log::set_max_level`] so that
    /// the `log` crate can discard records before they reach us.
    fn max_level(&self) -> LevelFilter {
        self.directives
            .iter()
            .map(|(_, level)| *level)
            .chain(std::iter::once(self.default))
            .max()
            .unwrap_or(LevelFilter::Off)
    }
}
//}}}
//{{{ fun: target_matches
/// Whether `target` is `filter` or a module path nested inside it.
///
/// Matching respects `::` boundaries, so `my_app` matches `my_app::solver` but not `my_apple`.
fn target_matches(
    target: &str,
    filter: &str,
) -> bool {
    if !target.starts_with(filter) {
        return false;
    }
    let rest = &target[filter.len()..];
    rest.is_empty() || rest.starts_with("::")
}
//}}}
//{{{ fun: parse_level
/// Parses a level name or its numeric shorthand.
fn parse_level(level: &str) -> Option<LevelFilter> {
    match level {
        "off" | "OFF" | "0" => Some(LevelFilter::Off),
        "error" | "ERROR" | "1" => Some(LevelFilter::Error),
        "warn" | "WARN" | "2" => Some(LevelFilter::Warn),
        "info" | "INFO" | "3" => Some(LevelFilter::Info),
        "debug" | "DEBUG" | "4" => Some(LevelFilter::Debug),
        "trace" | "TRACE" | "5" => Some(LevelFilter::Trace),
        _ => None,
    }
}
//}}}
//}}}
//{{{ collection: output sink
/// Where formatted records are written.
enum Sink {
    Stderr,
    Stdout,
    Writer(Mutex<Box<dyn Write + Send>>),
}

impl fmt::Debug for Sink {
    fn fmt(
        &self,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        match self {
            Self::Stderr => f.write_str("Stderr"),
            Self::Stdout => f.write_str("Stdout"),
            Self::Writer(_) => f.write_str("Writer(..)"),
        }
    }
}

impl Sink {
    /// Whether this sink is a terminal, and so whether colour is appropriate by default.
    fn is_terminal(&self) -> bool {
        match self {
            Self::Stderr => io::stderr().is_terminal(),
            Self::Stdout => io::stdout().is_terminal(),
            // A caller-supplied writer is assumed not to be a terminal; `Builder::color` can
            // override this.
            Self::Writer(_) => false,
        }
    }

    fn write_line(
        &self,
        line: &str,
    ) {
        // Logging must never take down the program, so write errors (a closed pipe, most often)
        // are dropped rather than propagated or panicked on.
        match self {
            Self::Stderr => drop(writeln!(io::stderr().lock(), "{line}")),
            Self::Stdout => drop(writeln!(io::stdout().lock(), "{line}")),
            Self::Writer(writer) => {
                let mut guard = writer.lock().unwrap_or_else(|e| e.into_inner());
                drop(writeln!(guard, "{line}"));
            }
        }
    }

    fn flush(&self) {
        match self {
            Self::Stderr => drop(io::stderr().flush()),
            Self::Stdout => drop(io::stdout().flush()),
            Self::Writer(writer) => {
                let mut guard = writer.lock().unwrap_or_else(|e| e.into_inner());
                drop(guard.flush());
            }
        }
    }
}
//}}}
//{{{ collection: Logger
//{{{ struct: Logger
/// The [`log`] backend installed by [`init`].
///
/// It is immutable once installed: filters are fixed at construction and indentation lives in
/// thread-local state, so logging takes no shared lock beyond the one the output stream imposes.
#[derive(Debug)]
struct Logger {
    filters: Filters,
    sink: Sink,
    color: bool,
}
//}}}
//{{{ impl log::Log for Logger
impl Log for Logger {
    fn enabled(
        &self,
        metadata: &Metadata<'_>,
    ) -> bool {
        metadata.level() <= self.filters.level_for(metadata.target())
    }

    fn log(
        &self,
        record: &Record<'_>,
    ) {
        if !self.enabled(record.metadata()) {
            return;
        }

        let level = record.level();
        let file = record.file().map(file_label).unwrap_or("<unknown>");
        let line = record.line().unwrap_or(0);

        // Pad before colouring: ANSI escapes count towards a format width, so colouring first
        // would make the level column wider than it looks and misalign coloured output against
        // uncoloured output.
        let level_field = format!("{:<5}", level.as_str());
        let level_field = if self.color {
            level_field.color(level_color(level)).to_string()
        } else {
            level_field
        };

        let location = format!("{file}:{line}");
        let padding = MESSAGE_COLUMN.saturating_sub(location.len()).max(1);
        let indent = INDENT.with(Cell::get) * INDENT_WIDTH;

        self.sink.write_line(&format!(
            "[{level_field}({}) {location}]{:padding$}{:indent$}{}",
            ThreadIdLabel(thread::current().id()),
            "",
            "",
            record.args(),
        ));
    }

    fn flush(&self) {
        self.sink.flush();
    }
}
//}}}
//{{{ fun: level_color
/// The colour each level is rendered in.
fn level_color(level: Level) -> &'static str {
    match level {
        Level::Error => "red",
        Level::Warn => "yellow",
        Level::Info => "green",
        Level::Debug => "blue",
        Level::Trace => "magenta",
    }
}
//}}}
//{{{ fun: file_label
/// Shortens a source path for display, keeping enough of it to stay unambiguous.
///
/// The file name is retained in full (including its extension) and, for the `mod.rs` and
/// `lib.rs`/`main.rs` cases where the name alone says nothing, the parent directory is kept too:
/// `src/parser/mod.rs` renders as `parser/mod.rs` rather than a bare `mod`.
fn file_label(full_path: &str) -> &str {
    let path = Path::new(full_path);
    let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
        return full_path;
    };

    if matches!(name, "mod.rs" | "lib.rs" | "main.rs") {
        if let Some(parent) = path
            .parent()
            .and_then(|p| p.file_name())
            .and_then(|p| p.to_str())
        {
            // Both components are suffixes of `full_path` and are separated by a single
            // separator, so slicing back from the file name yields `parent/name`.
            let start = full_path.len() - name.len() - parent.len() - 1;
            return &full_path[start..];
        }
    }

    name
}
//}}}
//{{{ struct: ThreadIdLabel
/// Renders a [`thread::ThreadId`] as its bare number.
///
/// `ThreadId` exposes its value only through `Debug` (`ThreadId(3)`) until `as_u64` stabilises, so
/// the number is extracted from that rendering.
struct ThreadIdLabel(thread::ThreadId);

impl fmt::Display for ThreadIdLabel {
    fn fmt(
        &self,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        let debug = format!("{:?}", self.0);
        let number = debug
            .find('(')
            .and_then(|start| {
                debug[start + 1..]
                    .find(')')
                    .map(|end| &debug[start + 1..start + 1 + end])
            })
            .unwrap_or("?");
        // `pad` honours the caller's width, fill and alignment, unlike a nested `format!`.
        f.pad(number)
    }
}
//}}}
//}}}
//{{{ collection: initialisation
//{{{ struct: Builder
/// Configures and installs the logger.
///
/// [`init`] is the shorthand for `Builder::new().init()`. Use the builder when filters should come
/// from somewhere other than the environment, or when output should not go to stderr:
///
/// ```no_run
/// use topohedral_tracing::Builder;
///
/// Builder::new()
///     .filters("all=warn,my_app::solver=trace")
///     .color(false)
///     .init()
///     .expect("tracing initialises once");
/// ```
#[derive(Debug)]
pub struct Builder {
    filters: Option<String>,
    sink: Sink,
    color: Option<bool>,
}

impl Default for Builder {
    fn default() -> Self {
        Self::new()
    }
}

impl Builder {
    /// Creates a builder that reads its filters from `TOPO_LOG`, writes to stderr, and colours
    /// output when stderr is a terminal.
    pub fn new() -> Self {
        Self {
            filters: None,
            sink: Sink::Stderr,
            color: None,
        }
    }

    /// Sets the filters explicitly, in `TOPO_LOG` syntax, instead of reading the environment.
    #[must_use]
    pub fn filters(
        mut self,
        spec: impl Into<String>,
    ) -> Self {
        self.filters = Some(spec.into());
        self
    }

    /// Writes records to stderr. This is the default.
    #[must_use]
    pub fn stderr(mut self) -> Self {
        self.sink = Sink::Stderr;
        self
    }

    /// Writes records to stdout.
    #[must_use]
    pub fn stdout(mut self) -> Self {
        self.sink = Sink::Stdout;
        self
    }

    /// Writes records to an arbitrary sink, such as a file or an in-memory buffer.
    ///
    /// Colour is off by default for a custom writer; call [`Builder::color`] to force it on.
    #[must_use]
    pub fn writer(
        mut self,
        writer: Box<dyn Write + Send>,
    ) -> Self {
        self.sink = Sink::Writer(Mutex::new(writer));
        self
    }

    /// Forces colour on or off, overriding terminal detection.
    ///
    /// By default colour is used when the chosen sink is a terminal and the `NO_COLOR`
    /// environment variable is unset.
    #[must_use]
    pub fn color(
        mut self,
        color: bool,
    ) -> Self {
        self.color = Some(color);
        self
    }

    /// Builds the logger and installs it as the global [`log`] backend.
    ///
    /// # Errors
    ///
    /// Returns [`TraceInitError::AlreadyInitialized`] if a global logger is already installed.
    /// Malformed filter directives are not an error: each is reported on stderr and skipped, so a
    /// typo in `TOPO_LOG` cannot silently disable logging or prevent startup.
    pub fn init(self) -> Result<(), TraceInitError> {
        let spec = match self.filters {
            Some(spec) => spec,
            None => match std::env::var("TOPO_LOG") {
                Ok(spec) => spec,
                Err(std::env::VarError::NotPresent) => String::new(),
                Err(std::env::VarError::NotUnicode(_)) => {
                    eprintln!("topohedral-tracing: TOPO_LOG is not valid unicode; ignoring it");
                    String::new()
                }
            },
        };

        let filters = Filters::parse(&spec, |problem| {
            eprintln!("topohedral-tracing: {problem}");
        });

        let color = self
            .color
            .unwrap_or_else(|| self.sink.is_terminal() && std::env::var_os("NO_COLOR").is_none());

        let max_level = filters.max_level();
        let logger = Logger {
            filters,
            sink: self.sink,
            color,
        };

        log::set_boxed_logger(Box::new(logger)).map_err(|_| TraceInitError::AlreadyInitialized)?;
        // Only now that we own the global logger is it ours to filter for.
        log::set_max_level(max_level);
        Ok(())
    }
}
//}}}
//{{{ fun: init
/// Initializes the tracing system, reading filters from `TOPO_LOG`.
///
/// This installs the global [`log`] backend and must be called before any tracing can occur,
/// typically from `main`. Records emitted before it returns are discarded.
///
/// Installing a global logger is a one-shot operation: a second call fails rather than
/// reconfiguring the first. Use [`Builder`] for anything beyond the defaults.
///
/// # Errors
///
/// Returns [`TraceInitError::AlreadyInitialized`] if a global logger is already installed, whether
/// by this crate or by another [`log`] backend.
pub fn init() -> Result<(), TraceInitError> {
    Builder::new().init()
}
//}}}
//}}}
//{{{ collection: scope tracing
//{{{ struct: IndentGuard
/// A guard that logs scope entry and exit and indents everything logged in between.
///
/// Created by [`trace_scope!`] and by the [`trace_fn`] attribute, which fill in the caller's
/// target and source location for you. Entry is logged and indentation raised when the guard is
/// constructed; indentation is lowered and exit logged when it is dropped, including when the
/// scope is left by `?` or by unwinding.
///
/// The guard is deliberately **not** [`Send`]: indentation is per-thread state, so a guard that
/// moved between threads would lower a different thread's indentation than it raised. This also
/// means it cannot be held across an `.await` in a task that migrates between executor threads.
#[derive(Debug)]
#[must_use = "the scope ends as soon as the guard is dropped; bind it to a variable"]
pub struct IndentGuard {
    name: Cow<'static, str>,
    target: &'static str,
    file: &'static str,
    line: u32,
    /// Makes the guard `!Send` and `!Sync`; see the type-level documentation.
    _not_send: PhantomData<*const ()>,
}

impl IndentGuard {
    /// Logs entry to a named scope and indents subsequent messages until the guard is dropped.
    ///
    /// `target` is the filter target for the entry and exit records, conventionally the calling
    /// module's `module_path!()`. Prefer [`trace_scope!`], which supplies it automatically — this
    /// constructor cannot determine the caller's module for itself.
    ///
    /// The source location is that of the caller, via `#[track_caller]`.
    #[track_caller]
    pub fn new(
        target: &'static str,
        name: impl Into<Cow<'static, str>>,
    ) -> Self {
        let location = std::panic::Location::caller();
        Self::with_location(target, name, location.file(), location.line())
    }

    /// As [`IndentGuard::new`], but with an explicit source location.
    ///
    /// Used by [`trace_scope!`], which captures `file!()` and `line!()` at the expansion site.
    #[doc(hidden)]
    pub fn with_location(
        target: &'static str,
        name: impl Into<Cow<'static, str>>,
        file: &'static str,
        line: u32,
    ) -> Self {
        let name = name.into();
        if ENABLED {
            __private::log_record(
                target,
                SCOPE_LEVEL,
                file,
                line,
                format_args!("* Entering {name}"),
            );
            increment_indent();
        }
        Self {
            name,
            target,
            file,
            line,
            _not_send: PhantomData,
        }
    }
}

impl Drop for IndentGuard {
    fn drop(&mut self) {
        if ENABLED {
            decrement_indent();
            __private::log_record(
                self.target,
                SCOPE_LEVEL,
                self.file,
                self.line,
                format_args!("* Leaving {}", self.name),
            );
        }
    }
}

/// The level at which scope entry and exit are logged.
const SCOPE_LEVEL: Level = Level::Info;
//}}}
//{{{ macro: trace_scope
/// Logs entry to and exit from the enclosing scope, indenting everything logged in between.
///
/// Expands to an [`IndentGuard`] bound to a local variable, so the scope ends when the enclosing
/// block does — including on an early return, a `?`, or a panic. Entry and exit are logged at
/// `info` against the calling module's target.
///
/// The name is anything convertible into `Cow<'static, str>`, so a string literal costs no
/// allocation and a computed name uses `format!`.
///
/// Only one invocation may appear per lexical scope: the guard is bound to a fixed internal name,
/// so a second would shadow the first. Use nested braces for several traced regions in one
/// function.
///
/// # Examples
///
/// ```no_run
/// use topohedral_tracing::{info, trace_scope};
///
/// fn my_function()
/// {
///     trace_scope!("my_function");
///     info!("this line is indented one level");
/// }
///
/// fn per_block(blocks: u32)
/// {
///     for block in 0..blocks
///     {
///         trace_scope!(format!("block {block}"));
///     }
/// }
/// ```
#[macro_export]
macro_rules! trace_scope {
    ($name:expr) => {
        let __topo_trace_guard = $crate::IndentGuard::with_location(
            ::core::module_path!(),
            $name,
            ::core::file!(),
            ::core::line!(),
        );
    };
}
//}}}
//}}}
//{{{ collection: logging macros
//{{{ macro: __topo_emit
/// Shared body of the five logging macros.
///
/// Gating on [`ENABLED`] — a constant of *this* crate — rather than on `#[cfg(feature = ..)]` is
/// what makes the compile-time switch depend on this crate's features rather than the calling
/// crate's. The `if` is folded away when the feature is off.
#[doc(hidden)]
#[macro_export]
macro_rules! __topo_emit {
    ($level:ident, target: $target:expr, $($arg:tt)+) => {
        if $crate::ENABLED {
            $crate::log::$level!(target: $target, $($arg)+);
        }
    };
    ($level:ident, $($arg:tt)+) => {
        if $crate::ENABLED {
            $crate::log::$level!($($arg)+);
        }
    };
}
//}}}
//{{{ macro: trace
/// Logs a message at the `trace` level, the most verbose.
///
/// Takes the same arguments as [`log::trace!`], including the optional `target:` prefix; without
/// one the target is the calling module's path.
///
/// ```no_run
/// use topohedral_tracing::trace;
///
/// trace!("entered with {} candidates", 3);
/// trace!(target: "solver", "entered with {} candidates", 3);
/// ```
#[macro_export]
macro_rules! trace {
    (target: $target:expr, $($arg:tt)+) => { $crate::__topo_emit!(trace, target: $target, $($arg)+) };
    ($($arg:tt)+) => { $crate::__topo_emit!(trace, $($arg)+) };
}
//}}}
//{{{ macro: debug
/// Logs a message at the `debug` level.
///
/// Takes the same arguments as [`log::debug!`]; see [`trace!`] for examples.
#[macro_export]
macro_rules! debug {
    (target: $target:expr, $($arg:tt)+) => { $crate::__topo_emit!(debug, target: $target, $($arg)+) };
    ($($arg:tt)+) => { $crate::__topo_emit!(debug, $($arg)+) };
}
//}}}
//{{{ macro: info
/// Logs a message at the `info` level.
///
/// Takes the same arguments as [`log::info!`]; see [`trace!`] for examples.
#[macro_export]
macro_rules! info {
    (target: $target:expr, $($arg:tt)+) => { $crate::__topo_emit!(info, target: $target, $($arg)+) };
    ($($arg:tt)+) => { $crate::__topo_emit!(info, $($arg)+) };
}
//}}}
//{{{ macro: warn
/// Logs a message at the `warn` level.
///
/// Takes the same arguments as [`log::warn!`]; see [`trace!`] for examples.
#[macro_export]
macro_rules! warn {
    (target: $target:expr, $($arg:tt)+) => { $crate::__topo_emit!(warn, target: $target, $($arg)+) };
    ($($arg:tt)+) => { $crate::__topo_emit!(warn, $($arg)+) };
}
//}}}
//{{{ macro: error
/// Logs a message at the `error` level, the least verbose.
///
/// Takes the same arguments as [`log::error!`]; see [`trace!`] for examples.
#[macro_export]
macro_rules! error {
    (target: $target:expr, $($arg:tt)+) => { $crate::__topo_emit!(error, target: $target, $($arg)+) };
    ($($arg:tt)+) => { $crate::__topo_emit!(error, $($arg)+) };
}
//}}}
//}}}
//{{{ collection: private
/// Implementation details used by this crate's macros.
///
/// Not public API: anything here may change or disappear in a patch release.
#[doc(hidden)]
pub mod __private {
    use super::*;

    /// Emits a record with an explicit source location.
    ///
    /// The logging macros go through [`log`]'s own macros, which capture the call site for us.
    /// This exists for [`IndentGuard`], whose entry and exit records must name the scope's
    /// location rather than this file's.
    pub fn log_record(
        target: &str,
        level: Level,
        file: &'static str,
        line: u32,
        args: fmt::Arguments<'_>,
    ) {
        if level > log::max_level() {
            return;
        }
        log::logger().log(
            &Record::builder()
                .args(args)
                .level(level)
                .target(target)
                .module_path(Some(target))
                .file(Some(file))
                .line(Some(line))
                .build(),
        );
    }
}
//}}}
//{{{ collection: tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn target_matching_respects_module_boundaries() {
        assert!(target_matches("my_app", "my_app"));
        assert!(target_matches("my_app::solver", "my_app"));
        assert!(target_matches("my_app::solver::inner", "my_app::solver"));
        assert!(!target_matches("my_apple", "my_app"));
        assert!(!target_matches("my_ap", "my_app"));
        assert!(!target_matches("other::my_app", "my_app"));
    }

    #[test]
    fn longest_matching_directive_wins() {
        let filters = Filters::parse("all=error,my_app=debug,my_app::solver=warn", |e| {
            panic!("unexpected parse error: {e}")
        });

        assert_eq!(filters.level_for("my_app"), LevelFilter::Debug);
        assert_eq!(filters.level_for("my_app::parser"), LevelFilter::Debug);
        // A more specific directive narrows a broader one; under the old `max`-with-`all`
        // semantics this was Debug.
        assert_eq!(filters.level_for("my_app::solver"), LevelFilter::Warn);
        assert_eq!(
            filters.level_for("my_app::solver::inner"),
            LevelFilter::Warn
        );
        assert_eq!(filters.level_for("unrelated"), LevelFilter::Error);
    }

    #[test]
    fn all_is_a_default_not_a_floor() {
        let filters = Filters::parse("all=debug,noisy=off", |e| panic!("unexpected: {e}"));
        assert_eq!(filters.level_for("noisy"), LevelFilter::Off);
        assert_eq!(filters.level_for("quiet"), LevelFilter::Debug);
    }

    #[test]
    fn bare_target_defaults_to_info_and_numbers_are_accepted() {
        let filters = Filters::parse("bare,numeric=5", |e| panic!("unexpected: {e}"));
        assert_eq!(filters.level_for("bare"), LevelFilter::Info);
        assert_eq!(filters.level_for("numeric"), LevelFilter::Trace);
    }

    #[test]
    fn malformed_directives_are_reported_and_skipped() {
        let mut problems = Vec::new();
        let filters = Filters::parse("good=debug,bad=nonsense,a=b=c,=empty", |e| problems.push(e));

        assert_eq!(problems.len(), 3, "got: {problems:?}");
        // The valid directive still takes effect.
        assert_eq!(filters.level_for("good"), LevelFilter::Debug);
    }

    #[test]
    fn empty_spec_disables_everything() {
        let filters = Filters::parse("", |e| panic!("unexpected: {e}"));
        assert_eq!(filters.level_for("anything"), LevelFilter::Off);
        assert_eq!(filters.max_level(), LevelFilter::Off);
    }

    #[test]
    fn max_level_covers_the_most_verbose_directive() {
        let filters = Filters::parse("all=warn,chatty=trace", |e| panic!("unexpected: {e}"));
        assert_eq!(filters.max_level(), LevelFilter::Trace);
    }

    #[test]
    fn file_label_keeps_enough_of_the_path_to_disambiguate() {
        assert_eq!(file_label("src/solver.rs"), "solver.rs");
        assert_eq!(file_label("src/parser/mod.rs"), "parser/mod.rs");
        assert_eq!(file_label("src/lib.rs"), "src/lib.rs");
        assert_eq!(file_label("solver.rs"), "solver.rs");
    }

    #[test]
    fn indentation_is_thread_local_and_saturates() {
        assert_eq!(indent_level(), 0);
        decrement_indent();
        assert_eq!(indent_level(), 0, "must saturate rather than underflow");

        increment_indent();
        increment_indent();
        assert_eq!(indent_level(), 2);

        let other = thread::spawn(indent_level).join().unwrap();
        assert_eq!(other, 0, "indentation must not leak across threads");

        decrement_indent();
        decrement_indent();
        assert_eq!(indent_level(), 0);
    }

    #[test]
    fn indent_guard_is_not_send() {
        // Compile-time assertion mirroring the `!Send` requirement; if `IndentGuard` ever gains
        // `Send` this stops compiling.
        trait AmbiguousIfSend<A> {
            fn maybe(&self) {}
        }
        impl<T: ?Sized> AmbiguousIfSend<()> for T {}
        struct Invalid;
        impl<T: ?Sized + Send> AmbiguousIfSend<Invalid> for T {}

        let guard = IndentGuard::with_location("tests", "scope", "lib.rs", 1);
        // Resolves unambiguously only because `IndentGuard: !Send`.
        guard.maybe();
    }
}
//}}}
