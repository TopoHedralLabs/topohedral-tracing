//! Tracing for the topohedral collection of crates.
//!
//! This crate provides a tracing mechanism for the topohedral collection of crates.
//! This also includes a feature ``enbale_trace`` which dependent crates can turn on to enable
//! tracing in the their code.
//--------------------------------------------------------------------------------------------------

#![feature(thread_id_value)]

//{{{ crate imports
//}}}
//{{{ std imports
use std::collections::HashMap;
use std::sync::Mutex;
use std::thread;
use std::fmt::Arguments;
//}}}
//{{{ dep imports
use colored::Colorize;
use log::{Level, LevelFilter, Metadata, Record, SetLoggerError};
//}}}
//--------------------------------------------------------------------------------------------------

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
                    if  key.contains("=") {

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
                    }
                    else {
                        target = key.to_string();
                        level = LevelFilter::Info;
                    }

                    if target == "all" {
                        all = level;
                    }
                    else {
                        filters.insert(target, level);
                    }
                }
            }
            Err(std::env::VarError::NotPresent) => {}
            Err(std::env::VarError::NotUnicode(_)) => {}
        }

        Self { filters: filters, all: all }
    }
}
//{{{ impl log::Log for TopoHedralLogger
impl log::Log for TopoHedralLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {

        let target = metadata.target();
        let mut target_level = match self.filters.get(target) {
            Some(level) => *level,
            None => self.all,
        };
        target_level = std::cmp::max(target_level, self.all);
        let is_enabled = metadata.level() <= target_level;
        is_enabled
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
pub fn init() -> Result<(), SetLoggerError> {
    let mut logger_guard = LOGGER.lock().unwrap();
    *logger_guard = Some(Box::new(TopoHedralLogger::new()));
    log::set_max_level(LevelFilter::Trace);
    // log::set_boxed_logger(logger_guard.take().unwrap())?;
    Ok(())
}
//}}}
//{{{ fun: topo_log
pub fn topo_log(target: &str, level: Level, module: &str, line: u32, args: Arguments) {
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
                    "[{} - Thread {} - {}:{}] {}",
                    level.as_str().color(log_color),
                    thread_id.as_u64(),
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

        std::env::set_var("TOPO_LOG", "test=5");
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
