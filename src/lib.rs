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
use std::{any::type_name, fmt::Arguments};
use std::sync::Mutex;
use std::thread;
use std::panic::Location;
//}}}
//{{{ dep imports
use log::{Level, LevelFilter, Metadata, Record, SetLoggerError};
use colored::Colorize;
//}}}
//--------------------------------------------------------------------------------------------------

//{{{ collection: constants
static LOGGER: Mutex<Option<Box<dyn log::Log>>> = Mutex::new(None);
//}}}
//{{{ collection TopoHedralLogger
//{{{ struct TopoHedralLogger
struct TopoHedralLogger;
//}}}
//{{{ impl TopoHedralLogger 
impl log::Log for TopoHedralLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Trace
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
    *logger_guard = Some(Box::new(TopoHedralLogger));
    log::set_max_level(LevelFilter::Trace);
    // log::set_boxed_logger(logger_guard.take().unwrap())?;
    Ok(())
}
//}}}
//{{{ macro: topo_log
// #[macro_export]
// macro_rules! topo_log {
//     ($target:expr, $level:expr, $($arg:tt)*) => {{

//         let mut logger_guard = LOGGER.lock().unwrap();
//         if let Some(logger) = &mut *logger_guard {
//             let location = Location::caller();
//             let function_name = location.file().rsplitn(2, '/').next().unwrap_or("");
//             let thread_id = thread::current().id();

//             let log_color = match $level{
//                 Level::Error => "red",
//                 Level::Warn => "yellow",
//                 Level::Info => "green",
//                 Level::Debug => "blue",
//                 Level::Trace => "magenta",
//             };

//             logger.log(&log::Record::builder()
//                 .args(format_args!("[{} - Thread {} - {}:{}] {}",
//                                     $level.as_str().color(log_color),
//                                     thread_id.as_u64(),
//                                     function_name,
//                                     location.line(), format_args!($($arg)*)))
//                 .level($level)
//                 .target($target)
//                 .build());
//         }

//     }}
// }
//}}}
//{{{ fun: topo_log
pub fn topo_log(target: &str, level: Level, location: &Location, args: Arguments) {

    let mut logger_guard = LOGGER.lock().unwrap();
    if let Some(logger) = &mut *logger_guard {

        let function_name = location.file().rsplitn(2, '/').next().unwrap_or("");
        let thread_id = thread::current().id();

        let log_color = match level{
            Level::Error => "red",
            Level::Warn => "yellow",
            Level::Info => "green",
            Level::Debug => "blue",
            Level::Trace => "magenta",
        };

        logger.log(&log::Record::builder()
            .args(format_args!("[{} - Thread {} - {}:{}] {}",
                                level.as_str().color(log_color),
                                thread_id.as_u64(),
                                function_name,
                                location.line(), 
                                args))
            .level(level)
            .target(target)
            .build());
    }
}
//}}}
//{{{ macro: trace
#[macro_export]
macro_rules! trace {
    (target: $target:expr, $($arg:tt)+) => {
        #[cfg(feature = "enable_trace")]
        topo_log($target, Level::Trace, Location::caller(), format_args!($($arg)+));
    };
    ($($arg:tt)+) => { 

        #[cfg(feature = "enable_trace")]
        {
            let location = Location::caller();
            let function_name = location.file().rsplitn(2, '/').next().unwrap_or("");
            topo_log(function_name, Level::Trace, location, format_args!($($arg)+));
        }
     };
}
//}}}
//{{{ macro: debug
#[macro_export]
macro_rules! debug{
    (target: $target:expr, $($arg:tt)+) => {
        #[cfg(feature = "enable_trace")]
        topo_log($target, Level::Debug, Location::caller(), format_args!($($arg)+));
    };
    ($($arg:tt)+) => { 

        #[cfg(feature = "enable_trace")]
        {
            let location = Location::caller();
            let function_name = location.file().rsplitn(2, '/').next().unwrap_or("");
            topo_log(function_name, Level::Debug, location, format_args!($($arg)+));
        }
     };
}
//}}}
//{{{ macro: info
#[macro_export]
macro_rules! info{
    (target: $target:expr, $($arg:tt)+) => {
        #[cfg(feature = "enable_trace")]
        topo_log($target, Level::Info, Location::caller(), format_args!($($arg)+));
    };
    ($($arg:tt)+) => { 

        #[cfg(feature = "enable_trace")]
        {
            let location = Location::caller();
            let function_name = location.file().rsplitn(2, '/').next().unwrap_or("");
            topo_log(function_name, Level::Info, location, format_args!($($arg)+));
        }
     };
}
//}}}
//{{{ macro: warn 
#[macro_export]
macro_rules! warn{
    (target: $target:expr, $($arg:tt)+) => {
        #[cfg(feature = "enable_trace")]
        topo_log($target, Level::Warn, Location::caller(), format_args!($($arg)+));
    };
    ($($arg:tt)+) => { 

        #[cfg(feature = "enable_trace")]
        {
            let location = Location::caller();
            let function_name = location.file().rsplitn(2, '/').next().unwrap_or("");
            topo_log(function_name, Level::Warn, location, format_args!($($arg)+));
        }
     };
}
//}}}
//{{{ macro: error 
#[macro_export]
macro_rules! error {
    (target: $target:expr, $($arg:tt)+) => {
        #[cfg(feature = "enable_trace")]
        topo_log($target, Level::Error, Location::caller(), format_args!($($arg)+));
    };
    ($($arg:tt)+) => { 

        #[cfg(feature = "enable_trace")]
        {
            let location = Location::caller();
            let function_name = location.file().rsplitn(2, '/').next().unwrap_or("");
            topo_log(function_name, Level::Error, location, format_args!($($arg)+));
        }
     };
}
//}}}


//-------------------------------------------------------------------------------------------------
//{{{ mod: tests
#[cfg(test)]
mod tests
{
  
    use super::*;

    #[test]
    fn test_topo_log() {
        init().unwrap();
        trace!("Hello, world! This is a test {}", 5);
        trace!(target: "test",  "Hello, world! This is a test {}", 5);
        debug!("Hello, world! This is a test {}", 5);
        debug!(target: "test",  "Hello, world! This is a test {}", 5);
        info!("Hello, world! This is a test {}", 5);
        info!(target: "test",  "Hello, world! This is a test {}", 5);
        warn!("Hello, world! This is a test {}", 5);
        warn!(target: "test",  "Hello, world! This is a test {}", 5);
        error!("Hello, world! This is a test {}", 5);
        error!(target: "test",  "Hello, world! This is a test {}", 5);
    }

}
//}}}
