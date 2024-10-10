//! Tracing for the topohedral collection of crates.
//!
//! # Introduction
//!
//! This crate provides a tracing mechanism for the topohedral collection of crates.
//! It is similar to the `env_logger` crate, and uses a similar syntax to specify the logging
//! filters. However it differs from `env_logger` in that it also provides a compile-time option to
//! enable or disable logging. It uses the `log` crate interface, therefore it provides five
//! macros for logging which are listed in decreasing order of verbosity:
//!
//! - `trace!`
//! - `debug!`
//! - `info!`
//! - `warn!`
//! - `error!`.
//!
//! # Usage
//!
//! ## Compile time configuration
//!
//! The printing code is only compiled if the `enable_trace` feature is enabled. Otherwise the
//! code resolves to nothing. Any crate which uses this crate can enable logging by having the
//! following in their `Cargo.toml` file:
//!
//! ``` toml
//! [dependencies]
//! topohedral-tracing = {<version etc>}
//!
//! [features]
//! enable_trace = ["topohedral-tracing/enable_trace"]
//! ```
//!
//! and compiling with the `enable_trace` feature.
//!
//! ## Runtime configuration
//!
//! Even with logging enabled at compile time, runtime logging filter will be dafault print
//! nothing. This can be changed by setting the `TOPO_LOG` environment variable. This variable
//! has the following syntax:
//!
//! ```shell
//! export TOPO_LOG=<target>=<level>,<target>=<level>,...
//! ```
//! Additionally, there is a special target `all` which can be used to enable all logging of a
//! given level. So, for example, to log everything at level `debug` we can do:
//!
//! ```shell
//! export TOPO_LOG=all=debug
//! ```
//!
//!
//--------------------------------------------------------------------------------------------------

//{{{ crate imports
//}}}
//{{{ std imports
use std::collections::HashMap;
use std::fmt;
use std::sync::Mutex;
use std::thread;
//}}}
//{{{ dep imports
use colored::Colorize;
use log::{Level, LevelFilter, Metadata, Record, SetLoggerError};
//}}}
//--------------------------------------------------------------------------------------------------
//{{{ impl fmt::Display for ThreadId
struct ThreadIdWrapper(thread::ThreadId);
impl fmt::Display for ThreadIdWrapper {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Use the Debug implementation to extract the number
        let thread_id_str = format!("{:?}", self.0);

        // Extract the number part from "ThreadId(num)"
        let num_str = if let Some(start) = thread_id_str.find('(') {
            if let Some(end) = thread_id_str.find(')') {
                &thread_id_str[start + 1..end]
            } else {
                "Unknown"
            }
        } else {
            "Unknown"
        };

        // Now format it respecting width and alignment
        f.write_str(&format!(
            "{:width$}",
            num_str,
            width = f.width().unwrap_or(0)
        ))
    }
}
//}}}
//{{{ collection: constants
static LOGGER: Mutex<Option<Box<dyn log::Log>>> = Mutex::new(None);
//}}}
//{{{ collection TopoHedralLogger
//{{{ struct TopoHedralLogger
struct TopoHedralLogger {
    all: LevelFilter,
    filters: HashMap<String, LevelFilter>,
}
//}}}
//{{{ impl TopoHedralLogger
impl TopoHedralLogger {
    fn new() -> Self {
        let mut filters = HashMap::<String, LevelFilter>::new();
        let mut all = LevelFilter::Off;

        match std::env::var("TOPO_LOG") {
            Ok(val) => {
                let targets: Vec<&str> = val.split(",").collect();
                for key in targets {
                    let target: String;
                    let level: LevelFilter;
                    if key.contains("=") {
                        let peices: Vec<&str> = key.split("=").collect();
                        target = peices[0].to_string();

                        level = match peices[1] {
                            "trace" | "5" => LevelFilter::Trace,
                            "debug" | "4" => LevelFilter::Debug,
                            "info" | "3" => LevelFilter::Info,
                            "warn" | "2" => LevelFilter::Warn,
                            "error" | "1" => LevelFilter::Error,
                            _ => LevelFilter::Info,
                        }
                    } else {
                        target = key.to_string();
                        level = LevelFilter::Info;
                    }

                    if target == "all" {
                        all = level;
                    } else {
                        filters.insert(target, level);
                    }
                }
            }
            Err(std::env::VarError::NotPresent) => {}
            Err(std::env::VarError::NotUnicode(_)) => {}
        }

        Self { filters, all }
    }
}
//}}}
//{{{ impl log::Log for TopoHedralLogger
impl log::Log for TopoHedralLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        let target = metadata.target();
        let mut target_level = match self.filters.get(target) {
            Some(level) => *level,
            None => self.all,
        };
        target_level = std::cmp::max(target_level, self.all);

        metadata.level() <= target_level
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            eprintln!("{}", record.args());
        }
    }

    fn flush(&self) {}
}
//}}}
//}}}
//{{{ fun: init
/// Initialize the tracing system.
///
/// This must be called before any tracing can occur. Typically this is called from the main
/// function of the program.
pub fn init() -> Result<(), SetLoggerError> {
    let mut logger_guard = LOGGER.lock().unwrap();
    *logger_guard = Some(Box::new(TopoHedralLogger::new()));
    log::set_max_level(LevelFilter::Trace);
    // log::set_boxed_logger(logger_guard.take().unwrap())?;
    Ok(())
}
//}}}
//{{{ fun: topo_log
/// Logs a message with the specified target, level, module, line, and arguments.
///
/// This function is used internally by the `trace!`, `debug!`, and `info!` macros to log
/// messages with the appropriate metadata. It acquires a lock on the `LOGGER` global
/// variable, and if a logger is present, it logs the message with the specified
/// attributes.
///
/// The `log_color` variable is used to determine the color of the log message based on
/// the log level. The message is then formatted and logged using the logger's `log`
/// method.
///
/// # Arguments
/// - target: &str - The target of the log message.
/// - level: Level - The level of the log message.
/// - module: &str - The module of the log message.
/// - line: u32 - The line of the log message.
/// - args: Arguments - The arguments of the log message.
pub fn topo_log(target: &str, level: Level, module: &str, line: u32, args: fmt::Arguments) {
    let mut logger_guard = LOGGER.lock().unwrap();
    if let Some(logger) = &mut *logger_guard {
        let thread_id = thread::current().id();

        let log_color = match level {
            Level::Error => "red",
            Level::Warn => "yellow",
            Level::Info => "green",
            Level::Debug => "blue",
            Level::Trace => "magenta",
        };

        logger.log(
            &log::Record::builder()
                .args(format_args!(
                    "[{:<5} - {:<3} - {}:{}] {}",
                    level.as_str().color(log_color),
                    ThreadIdWrapper(thread_id),
                    module,
                    line,
                    args
                ))
                .file(Some(module))
                .line(Some(line))
                .level(level)
                .target(target)
                .build(),
        );
    }
}
//}}}
//{{{ macro: trace
/// The `trace!` macro is used to log a trace message. Trace is the highest level of logging.
#[macro_export]
macro_rules! trace {
    (target: $target:expr, $($arg:tt)+) => {
        #[cfg(feature = "enable_trace")]
        {
            let location = std::panic::Location::caller();
            let module = module_path!();
            topo_log($target, log::Level::Trace, module, location.line(), format_args!($($arg)+));
        }
    };
    ($($arg:tt)+) => {

        #[cfg(feature = "enable_trace")]
        {
            let location = std::panic::Location::caller();
            let module = module_path!();
            topo_log(module, log::Level::Trace, module, location.line(), format_args!($($arg)+));
        }
     };
}
//}}}
//{{{ macro: debug
///  The `debug!` macro is used to log a debug message. Debug is the second highest level of logging.
#[macro_export]
macro_rules! debug{
    (target: $target:expr, $($arg:tt)+) => {
        #[cfg(feature = "enable_trace")]
        {
            let location = std::panic::Location::caller();
            let module = module_path!();
            topo_log($target, log::Level::Debug, module, location.line(), format_args!($($arg)+));
        }
    };
    ($($arg:tt)+) => {

        #[cfg(feature = "enable_trace")]
        {
            let location = std::panic::Location::caller();
            let module = module_path!();
            topo_log(module, log::Level::Debug, module, location.line(), format_args!($($arg)+));
        }
     };
}
//}}}
//{{{ macro: info
/// The `info!` macro is used to log an info message. Info is the third highest level of logging.
#[macro_export]
macro_rules! info{
    (target: $target:expr, $($arg:tt)+) => {
        #[cfg(feature = "enable_trace")]
        {
            let location = std::panic::Location::caller();
            let module = module_path!();
            topo_log($target, log::Level::Info, module, location.line(), format_args!($($arg)+));
        }
    };
    ($($arg:tt)+) => {

        #[cfg(feature = "enable_trace")]
        {
            let location = std::panic::Location::caller();
            let module = module_path!();
            topo_log(module, log::Level::Info, module, location.line(), format_args!($($arg)+));
        }
     };
}
//}}}
//{{{ macro: warn
/// The `warn!` macro is used to log a warning message. Warn is the fourth highest level of logging.
#[macro_export]
macro_rules! warn{
    (target: $target:expr, $($arg:tt)+) => {
        #[cfg(feature = "enable_trace")]
        {
            let location = std::panic::Location::caller();
            let module = module_path!();
            topo_log($target, log::Level::Warn, module, location.line(), format_args!($($arg)+));
        }
    };
    ($($arg:tt)+) => {

        #[cfg(feature = "enable_trace")]
        {
            let location = std::panic::Location::caller();
            let module = module_path!();
            topo_log(module, log::Level::Warn, module, location.line(), format_args!($($arg)+));
        }
     };
}
//}}}
//{{{ macro: error
/// The `error!` macro is used to log an error message. Error is the lowest level of logging.
#[macro_export]
macro_rules! error {
    (target: $target:expr, $($arg:tt)+) => {
        #[cfg(feature = "enable_trace")]
        {
            let location = std::panic::Location::caller();
            let module = module_path!();
            topo_log($target, log::Level::Error, module, location.line(), format_args!($($arg)+));
        }
    };
    ($($arg:tt)+) => {

        #[cfg(feature = "enable_trace")]
        {
            let location = std::panic::Location::caller();
            let module = module_path!();
            topo_log(module, log::Level::Error, module, location.line(), format_args!($($arg)+));
        }
     };
}
//}}}
//-------------------------------------------------------------------------------------------------
//{{{ mod: tests
#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_topo_log() {
        std::env::set_var("TOPO_LOG", "all=5");
        init().unwrap();
        trace!("Hello, world! This is a test 1 {}", 5);
        trace!(target: "test",  "Hello, world! This is a test 2 {}", 5);
        debug!("Hello, world! This is a test 1 {}", 5);
        debug!(target: "test",  "Hello, world! This is a test 2 {}", 5);
        info!("Hello, world! This is a test 1 {}", 5);
        info!(target: "test",  "Hello, world! This is a test 2 {}", 5);
        warn!("Hello, world! This is a test 1 {}", 5);
        warn!(target: "test",  "Hello, world! This is a test 2 {}", 5);
        error!("Hello, world! This is a test 1 {}", 5);
        error!(target: "test",  "Hello, world! This is a test 2 {}", 5);
    }
}
//}}}
